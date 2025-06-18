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

/// 进度回调函数类型 (completed, total)
pub type ProgressCallback = Arc<dyn Fn(usize, usize) + Send + Sync>;

/// 调度器配置
#[derive(Clone)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrency: usize,
    /// 任务超时时长（None 表示不限制）
    pub timeout: Option<Duration>,
    /// 是否在第一个任务失败时停止所有任务
    pub fail_fast: bool,
    /// 是否显示详细日志
    pub verbose: bool,
    /// 进度回调函数 (completed, total)
    pub progress_callback: Option<ProgressCallback>,
    /// 任务完成回调函数
    pub task_completed_callback: Option<Arc<dyn Fn(&str, &TaskResult<()>) + Send + Sync>>,
}

impl std::fmt::Debug for SchedulerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchedulerConfig")
            .field("max_concurrency", &self.max_concurrency)
            .field("timeout", &self.timeout)
            .field("fail_fast", &self.fail_fast)
            .field("verbose", &self.verbose)
            .field("has_progress_callback", &self.progress_callback.is_some())
            .field(
                "has_task_completed_callback",
                &self.task_completed_callback.is_some(),
            )
            .finish()
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrency: num_cpus::get(),
            timeout: None,
            fail_fast: false,
            verbose: false,
            progress_callback: None,
            task_completed_callback: None,
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
    /// 已完成任务计数
    completed_count: Arc<RwLock<usize>>,
    /// 成功任务计数
    successful_count: Arc<RwLock<usize>>,
    /// 失败任务计数
    failed_count: Arc<RwLock<usize>>,
}

impl AsyncTaskScheduler {
    /// 创建新的调度器
    pub fn new(config: SchedulerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        let task_status = Arc::new(RwLock::new(HashMap::new()));
        let should_stop = Arc::new(RwLock::new(false));
        let completed_count = Arc::new(RwLock::new(0));
        let successful_count = Arc::new(RwLock::new(0));
        let failed_count = Arc::new(RwLock::new(0));

        Self {
            config,
            semaphore,
            task_status,
            should_stop,
            completed_count,
            successful_count,
            failed_count,
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

        // 更新计数器并调用进度回调
        self.update_counters_and_progress(is_success).await;

        // 调用任务完成回调
        if let Some(callback) = &self.config.task_completed_callback {
            // 对于泛型结果，我们创建一个简化的 TaskResult<()>
            let simple_result = match &result {
                TaskResult::Success(_) => TaskResult::Success(()),
                TaskResult::Failed(err) => TaskResult::Failed(err.clone()),
                TaskResult::Timeout => TaskResult::Timeout,
                TaskResult::Cancelled => TaskResult::Cancelled,
            };
            callback(&task_id, &simple_result);
        }

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

        // 重置停止标志和计数器
        *self.should_stop.write().await = false;
        *self.completed_count.write().await = 0;
        *self.successful_count.write().await = 0;
        *self.failed_count.write().await = 0;

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

    /// 专门用于依赖检查的简化接口
    pub async fn execute_dependency_checks<F>(
        &self,
        dependencies: Vec<(String, F)>,
    ) -> HashMap<String, TaskResult<()>>
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let results = self.execute_batch(dependencies).await;

        // 转换为 HashMap 便于查找
        results
            .into_iter()
            .collect::<HashMap<String, TaskResult<()>>>()
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

    /// 获取当前执行进度
    pub async fn get_progress(&self) -> (usize, usize) {
        let completed = *self.completed_count.read().await;
        let total = self.task_status.read().await.len();

        (completed, total)
    }

    /// 获取详细执行统计
    pub async fn get_detailed_progress(&self) -> (usize, usize, usize, usize) {
        let completed = *self.completed_count.read().await;
        let total = self.task_status.read().await.len();
        let successful = *self.successful_count.read().await;
        let failed = *self.failed_count.read().await;

        (completed, total, successful, failed)
    }

    /// 设置进度回调函数
    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.config.progress_callback = Some(callback);
        self
    }

    /// 设置任务完成回调函数
    pub fn with_task_completed_callback(
        mut self,
        callback: Arc<dyn Fn(&str, &TaskResult<()>) + Send + Sync>,
    ) -> Self {
        self.config.task_completed_callback = Some(callback);
        self
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

    /// 更新计数器并调用进度回调
    async fn update_counters_and_progress(&self, is_success: bool) {
        // 更新计数器
        {
            let mut completed = self.completed_count.write().await;
            *completed += 1;
        }

        if is_success {
            let mut successful = self.successful_count.write().await;
            *successful += 1;
        } else {
            let mut failed = self.failed_count.write().await;
            *failed += 1;
        }

        // 调用进度回调
        if let Some(callback) = &self.config.progress_callback {
            let completed = *self.completed_count.read().await;
            let total = self.task_status.read().await.len();

            callback(completed, total);
        }
    }

    /// 为任务执行创建调度器克隆
    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            semaphore: Arc::clone(&self.semaphore),
            task_status: Arc::clone(&self.task_status),
            should_stop: Arc::clone(&self.should_stop),
            completed_count: Arc::clone(&self.completed_count),
            successful_count: Arc::clone(&self.successful_count),
            failed_count: Arc::clone(&self.failed_count),
        }
    }
}
