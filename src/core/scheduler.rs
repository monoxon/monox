// ============================================================================
// MonoX - 异步任务调度器
// ============================================================================
//
// 文件: src/core/scheduler.rs
// 职责: 通用异步任务调度和并发控制
// 边界:
//   - ✅ 异步任务调度和执行
//   - ✅ 并发数量控制
//   - ✅ 任务超时管理
//   - ✅ 执行结果聚合
//   - ✅ 错误处理和传播
//   - ✅ 通用 Future 执行支持
//   - ❌ 不包含具体业务逻辑
//   - ❌ 不包含命令执行细节
//   - ❌ 不包含 UI 显示逻辑
//   - ❌ 不包含配置管理
//
// ============================================================================

use crate::utils::logger::Logger;
use crate::{t, tf};
use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::timeout;

/// 任务执行结果枚举
#[derive(Debug, Clone)]
pub enum TaskResult<T> {
    /// 任务执行成功
    Success(T),
    /// 任务执行失败
    Failed(String),
    /// 任务执行超时
    Timeout,
    /// 任务被取消
    Cancelled,
}

/// 任务状态信息
#[derive(Debug, Clone)]
pub struct TaskStatus {
    /// 任务ID
    pub id: String,
    /// 任务开始时间
    pub started_at: Instant,
    /// 任务结束时间
    pub completed_at: Option<Instant>,
    /// 任务是否完成
    pub is_completed: bool,
    /// 任务是否成功
    pub is_success: bool,
}

/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrency: usize,
    /// 任务超时时长（None 表示不限制）
    pub timeout: Option<Duration>,
    /// 是否在第一个任务失败时停止所有任务
    pub fail_fast: bool,
    /// 是否显示详细日志
    pub verbose: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrency: num_cpus::get(),
            timeout: None,
            fail_fast: false,
            verbose: false,
        }
    }
}

/// 异步任务调度器
pub struct AsyncTaskScheduler {
    /// 调度器配置
    config: SchedulerConfig,
    /// 并发控制信号量
    semaphore: Arc<Semaphore>,
    /// 任务状态追踪
    task_status: Arc<RwLock<HashMap<String, TaskStatus>>>,
    /// 是否应该停止执行
    should_stop: Arc<RwLock<bool>>,
}

impl AsyncTaskScheduler {
    /// 创建新的调度器
    pub fn new(config: SchedulerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        let task_status = Arc::new(RwLock::new(HashMap::new()));
        let should_stop = Arc::new(RwLock::new(false));

        Self {
            config,
            semaphore,
            task_status,
            should_stop,
        }
    }

    /// 执行单个异步任务
    pub async fn execute_task<T, F>(&self, task_id: String, task: F) -> TaskResult<T>
    where
        T: Send + 'static,
        F: Future<Output = Result<T>> + Send + 'static,
    {
        // 检查是否应该停止
        if *self.should_stop.read().await {
            return TaskResult::Cancelled;
        }

        // 获取信号量许可
        let _permit = match self.semaphore.acquire().await {
            Ok(permit) => permit,
            Err(_) => return TaskResult::Cancelled,
        };

        // 记录任务开始
        let start_time = Instant::now();
        self.record_task_start(&task_id, start_time).await;

        if self.config.verbose {
            Logger::info(tf!("scheduler.task_start", &task_id));
        }

        // 执行任务（可能有超时）
        let result = match self.config.timeout {
            Some(timeout_duration) => match timeout(timeout_duration, task).await {
                Ok(task_result) => match task_result {
                    Ok(value) => TaskResult::Success(value),
                    Err(e) => TaskResult::Failed(e.to_string()),
                },
                Err(_) => TaskResult::Timeout,
            },
            None => match task.await {
                Ok(value) => TaskResult::Success(value),
                Err(e) => TaskResult::Failed(e.to_string()),
            },
        };

        // 记录任务完成
        let is_success = matches!(result, TaskResult::Success(_));
        self.record_task_completion(&task_id, is_success).await;

        // 如果配置了 fail_fast 且任务失败，则停止所有其他任务
        if self.config.fail_fast && !is_success {
            *self.should_stop.write().await = true;
            if self.config.verbose {
                Logger::warn(tf!("scheduler.fail_fast_triggered", &task_id));
            }
        }

        // 输出任务结果日志
        if self.config.verbose {
            let duration = start_time.elapsed();
            match &result {
                TaskResult::Success(_) => {
                    Logger::info(tf!(
                        "scheduler.task_success",
                        &task_id,
                        duration.as_secs_f64()
                    ));
                }
                TaskResult::Failed(err) => {
                    Logger::error(tf!(
                        "scheduler.task_failed",
                        &task_id,
                        duration.as_secs_f64(),
                        err
                    ));
                }
                TaskResult::Timeout => {
                    Logger::warn(tf!(
                        "scheduler.task_timeout",
                        &task_id,
                        duration.as_secs_f64()
                    ));
                }
                TaskResult::Cancelled => {
                    Logger::warn(tf!("scheduler.task_cancelled", &task_id));
                }
            }
        }

        result
    }

    /// 并发执行多个任务
    pub async fn execute_batch<T, F>(&self, tasks: Vec<(String, F)>) -> Vec<(String, TaskResult<T>)>
    where
        T: Send + 'static,
        F: Future<Output = Result<T>> + Send + 'static,
    {
        if tasks.is_empty() {
            return Vec::new();
        }

        if self.config.verbose {
            Logger::info(tf!("scheduler.batch_start", tasks.len()));
        }

        // 重置停止标志
        *self.should_stop.write().await = false;

        // 创建任务句柄
        let mut handles: Vec<JoinHandle<(String, TaskResult<T>)>> = Vec::new();

        for (task_id, task) in tasks {
            let scheduler = self.clone_for_task();
            let task_id_clone = task_id.clone();

            let handle = tokio::spawn(async move {
                let result = scheduler.execute_task(task_id_clone.clone(), task).await;
                (task_id_clone, result)
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((task_id, result)) => results.push((task_id, result)),
                Err(e) => {
                    Logger::error(tf!("scheduler.task_join_error", e.to_string()));
                }
            }
        }

        if self.config.verbose {
            let success_count = results
                .iter()
                .filter(|(_, result)| matches!(result, TaskResult::Success(_)))
                .count();
            let total_count = results.len();

            Logger::info(tf!("scheduler.batch_complete", success_count, total_count));
        }

        results
    }

    /// 获取任务状态快照
    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        self.task_status.read().await.get(task_id).cloned()
    }

    /// 获取所有任务状态
    pub async fn get_all_task_status(&self) -> HashMap<String, TaskStatus> {
        self.task_status.read().await.clone()
    }

    /// 检查是否有任务正在运行
    pub async fn has_running_tasks(&self) -> bool {
        self.task_status
            .read()
            .await
            .values()
            .any(|status| !status.is_completed)
    }

    /// 停止所有正在执行的任务
    pub async fn stop_all(&self) {
        *self.should_stop.write().await = true;
        if self.config.verbose {
            Logger::warn(t!("scheduler.stopping_all_tasks"));
        }
    }

    /// 获取执行统计信息
    pub async fn get_execution_summary(&self) -> ExecutionSummary {
        let task_status = self.task_status.read().await;
        let total_tasks = task_status.len();
        let completed_tasks = task_status.values().filter(|s| s.is_completed).count();
        let successful_tasks = task_status.values().filter(|s| s.is_success).count();
        let failed_tasks = completed_tasks - successful_tasks;

        let total_duration = task_status
            .values()
            .filter_map(|s| s.completed_at.map(|end| end.duration_since(s.started_at)))
            .max()
            .unwrap_or(Duration::from_secs(0));

        ExecutionSummary {
            total_tasks,
            completed_tasks,
            successful_tasks,
            failed_tasks,
            total_duration,
        }
    }

    /// 记录任务开始
    async fn record_task_start(&self, task_id: &str, start_time: Instant) {
        let status = TaskStatus {
            id: task_id.to_string(),
            started_at: start_time,
            completed_at: None,
            is_completed: false,
            is_success: false,
        };

        self.task_status
            .write()
            .await
            .insert(task_id.to_string(), status);
    }

    /// 记录任务完成
    async fn record_task_completion(&self, task_id: &str, is_success: bool) {
        if let Some(status) = self.task_status.write().await.get_mut(task_id) {
            status.completed_at = Some(Instant::now());
            status.is_completed = true;
            status.is_success = is_success;
        }
    }

    /// 为任务执行创建调度器克隆
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            semaphore: Arc::clone(&self.semaphore),
            task_status: Arc::clone(&self.task_status),
            should_stop: Arc::clone(&self.should_stop),
        }
    }
}

/// 执行统计摘要
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    /// 总任务数
    pub total_tasks: usize,
    /// 完成的任务数
    pub completed_tasks: usize,
    /// 成功的任务数
    pub successful_tasks: usize,
    /// 失败的任务数
    pub failed_tasks: usize,
    /// 总执行时长
    pub total_duration: Duration,
}

impl ExecutionSummary {
    /// 打印执行摘要
    pub fn print_summary(&self) {
        use crate::utils::constants::icons;

        Logger::info(format!(
            "\n{} {}",
            icons::SUMMARY,
            t!("scheduler.execution_summary")
        ));
        Logger::info("═══════════════════════════════════════");

        Logger::info(tf!("scheduler.summary_total", self.total_tasks));
        Logger::info(tf!("scheduler.summary_completed", self.completed_tasks));
        Logger::info(tf!("scheduler.summary_successful", self.successful_tasks));

        if self.failed_tasks > 0 {
            Logger::error(tf!("scheduler.summary_failed", self.failed_tasks));
        }

        Logger::info(tf!(
            "scheduler.summary_duration",
            self.total_duration.as_secs_f64()
        ));

        if self.failed_tasks == 0 && self.completed_tasks == self.total_tasks {
            Logger::success(format!(
                "{} {}",
                icons::SUCCESS,
                t!("scheduler.all_success")
            ));
        } else if self.successful_tasks > 0 {
            Logger::warn(format!(
                "{} {}",
                icons::WARNING,
                t!("scheduler.partial_success")
            ));
        } else {
            Logger::error(format!("{} {}", icons::ERROR, t!("scheduler.all_failed")));
        }
    }
}
