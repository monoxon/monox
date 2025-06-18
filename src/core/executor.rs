// ============================================================================
// MonoX - 任务执行器
// ============================================================================
//
// 文件: src/core/executor.rs
// 职责: 基础任务执行功能
// 边界:
//   - ✅ 单个任务执行
//   - ✅ 命令执行和输出捕获
//   - ✅ 任务状态管理
//   - ✅ 跨平台命令检测
//   - ❌ 不包含并发执行逻辑
//   - ❌ 不包含阶段化执行逻辑
//   - ❌ 不包含依赖分析逻辑
//   - ❌ 不包含 CLI 参数处理
//
// ============================================================================

use crate::core::{AsyncTaskScheduler, DependencyAnalyzer, SchedulerConfig, SchedulerTaskResult};
use crate::models::config::Config;
use crate::models::package::WorkspacePackage;
use crate::models::{Task, TaskConfig, TaskResult, TaskStatus};
use crate::utils::constants::icons;
use crate::utils::logger::Logger;
use crate::{t, tf};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// 执行命令并返回结果
async fn run_command(task: &Task) -> Result<TaskResult> {
    let start_time = Instant::now();

    let package_manager = Config::get_package_manager().as_str();
    let command_str = &format!("{} run {}", package_manager, task.command);

    // 构建命令
    let mut command = Command::new(package_manager);
    command.arg("run").arg(&task.command);

    // 执行命令目录
    let working_directory = Config::get_workspace_root().join(&task.working_directory);

    command
        .args(&task.args)
        .current_dir(&working_directory)
        .envs(&task.env_vars)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if Config::get_verbose() {
        Logger::info(tf!(
            "executor.command_run",
            &task.command,
            task.args.join(" ")
        ));
    }

    // 执行命令
    let output = command
        .output()
        .context(tf!("executor.command_failed", command_str).to_string())?;

    let duration = start_time.elapsed();
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    // 在详细模式下输出命令输出
    if Config::get_verbose() {
        if !stdout.is_empty() {
            Logger::info(tf!("executor.command_stdout", &stdout));
        }
        if !stderr.is_empty() {
            Logger::warn(tf!("executor.command_stderr", &stderr));
        }
    }

    // 创建任务结果
    let result = if success {
        TaskResult::success(stdout, duration)
    } else {
        TaskResult::failure(exit_code, stdout, stderr, duration)
    };

    Ok(result)
}

/// 执行单个任务
async fn execute_task(task: &mut Task) -> Result<()> {
    Logger::info(tf!(
        "executor.task_start",
        &task.package_name,
        &task.command
    ));

    // 开始执行
    task.start();

    if task.status == TaskStatus::Skipped {
        Logger::warn(tf!(
            "executor.task_skipped",
            &task.package_name,
            &task.command
        ));
        return Ok(());
    }

    let start_time = Instant::now();

    // 执行命令
    let result = run_command(task).await?;

    // 更新任务状态
    task.complete(result);

    // 输出结果
    if task.is_success() {
        Logger::success(tf!(
            "executor.task_success",
            &task.package_name,
            &task.command,
            start_time.elapsed().as_secs_f64()
        ));
    } else {
        Logger::error(tf!(
            "executor.task_failed",
            &task.package_name,
            &task.command,
            start_time.elapsed().as_secs_f64()
        ));

        if let Some(task_result) = &task.result {
            if !task_result.stderr.is_empty() {
                Logger::error(tf!("executor.task_stderr", &task_result.stderr));
            }
        }
    }

    Ok(())
}

/// 基础任务执行器
pub struct TaskExecutor {
    /// 任务配置
    config: TaskConfig,
}

impl TaskExecutor {
    /// 创建新的任务执行器
    pub fn new(config: TaskConfig) -> Self {
        Self { config }
    }

    /// 从全局配置创建任务执行器
    pub fn new_from_config() -> Result<Self> {
        let config = TaskConfig {
            max_concurrency: Config::get_max_concurrency(),
            verbose: Config::get_verbose(),
            ..Default::default()
        };
        Ok(Self { config })
    }

    /// 通用执行方法，支持 run 和 exec 两种调用方式
    pub async fn execute(
        &self,
        package_name: &str,
        command: &str,
        all: Option<bool>,
    ) -> Result<()> {
        match (all.unwrap_or(false), package_name) {
            // all 为 true 时，执行所有包
            (true, _) => self.execute_all_packages(command).await,
            // all 为 false，且有 package_name 时，执行单包
            (false, pkg_name) => self.execute_single_package(pkg_name, command).await,
        }
    }

    /// 执行所有包（all = true）
    async fn execute_all_packages(&self, command: &str) -> Result<()> {
        // 获取工作区根目录（从全局配置中获取）
        let workspace_root = Config::get_workspace_root();
        // 创建分析器，获取包信息
        let mut analyzer =
            DependencyAnalyzer::new(workspace_root.to_path_buf()).with_verbose(self.config.verbose);
        let analysis_result = analyzer.analyze_workspace()?;

        Logger::info(t!("run.scanning_all_packages"));

        // 检查包是否有指定的脚本
        let executable_packages: Vec<_> = analysis_result
            .packages
            .into_iter()
            .filter(|package| {
                if package.scripts.contains_key(command) {
                    true
                } else {
                    if self.config.verbose {
                        Logger::warn(tf!("run.script_not_found", &package.name, command));
                    }
                    false
                }
            })
            .collect();

        if executable_packages.is_empty() {
            anyhow::bail!(tf!("run.no_executable_packages", command));
        }

        Logger::info(tf!(
            "run.found_executable_packages",
            executable_packages.len(),
            command
        ));

        self.execute_stages(&analysis_result.stages, command).await
    }

    /// 执行单个包
    async fn execute_single_package(&self, package_name: &str, command: &str) -> Result<()> {
        // 获取工作区根目录（从全局配置中获取）
        let workspace_root = Config::get_workspace_root();
        // 创建分析器，获取包信息
        let mut analyzer =
            DependencyAnalyzer::new(workspace_root.to_path_buf()).with_verbose(self.config.verbose);
        let analysis_result = analyzer.analyze_single_package(package_name)?;

        // 查找指定的包
        let package = analysis_result
            .packages
            .iter()
            .find(|p| p.name == package_name)
            .ok_or_else(|| anyhow::anyhow!(tf!("run.package_not_found", package_name)))?;

        // 检查包是否有指定的脚本
        if !package.scripts.contains_key(command) {
            anyhow::bail!(tf!("run.script_not_found", package_name, command));
        }

        Logger::info(tf!("run.found_executable_packages", 1, command));

        self.execute_stages(&analysis_result.stages, command).await
    }

    /// 执行阶段任务
    async fn execute_stages(
        &self,
        stages: &Vec<Vec<WorkspacePackage>>,
        command: &str,
    ) -> Result<()> {
        for stage in stages {
            self.execute_single_stage(stage, command).await?;
        }
        Ok(())
    }

    /// 单个阶段任务
    async fn execute_single_stage(
        &self,
        stage: &Vec<WorkspacePackage>,
        command: &str,
    ) -> Result<()> {
        if stage.is_empty() {
            return Ok(());
        }

        // 单个包时保持原有串行逻辑，避免异步开销
        if stage.len() == 1 {
            let package = &stage[0];
            let mut task = Task::new(
                package.name.clone(),
                package.folder.to_string_lossy().to_string(),
                command.to_string(),
                vec![],
            );
            return execute_task(&mut task).await;
        }

        // 多个包时使用并发执行
        Logger::info(tf!("executor.stage_concurrent_start", stage.len()));

        // 创建调度器配置
        let scheduler_config = SchedulerConfig {
            max_concurrency: self.config.max_concurrency,
            timeout: self
                .config
                .timeout_seconds
                .map(|s| Duration::from_secs(s as u64)),
            fail_fast: !self.config.continue_on_error,
            verbose: self.config.verbose,
        };

        let scheduler = AsyncTaskScheduler::new(scheduler_config);

        // 准备异步任务
        let tasks: Vec<(String, _)> = stage
            .iter()
            .map(|package| {
                let task_id = format!("{}:{}", package.name, command);

                let mut task = Task::new(
                    package.name.clone(),
                    package.folder.to_string_lossy().to_string(),
                    command.to_string(),
                    vec![],
                );

                let task_future = async move { execute_task(&mut task).await };

                (task_id, task_future)
            })
            .collect();

        // 在同步上下文中运行异步代码
        let results = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { scheduler.execute_batch(tasks).await })
        });

        // 处理执行结果
        let mut success_count = 0;
        let mut failed_tasks = Vec::new();

        for (task_id, result) in results {
            match result {
                SchedulerTaskResult::Success(_) => {
                    success_count += 1;
                    if self.config.verbose {
                        Logger::success(tf!("executor.task_concurrent_success", &task_id));
                    }
                }
                SchedulerTaskResult::Failed(err) => {
                    failed_tasks.push(format!("{}: {}", task_id, err));
                    Logger::error(tf!("executor.task_concurrent_failed", &task_id, &err));
                }
                SchedulerTaskResult::Timeout => {
                    failed_tasks.push(format!("{}: 执行超时", task_id));
                    Logger::error(tf!("executor.task_concurrent_timeout", &task_id));
                }
                SchedulerTaskResult::Cancelled => {
                    Logger::warn(tf!("executor.task_concurrent_cancelled", &task_id));
                }
            }
        }

        Logger::info(tf!(
            "executor.stage_concurrent_complete",
            success_count,
            stage.len()
        ));

        // 如果有失败任务且不允许继续执行，则返回错误
        if !failed_tasks.is_empty() {
            anyhow::bail!("阶段执行失败: {}", failed_tasks.join(", "));
        }

        Ok(())
    }

    /// 打印执行汇总
    fn print_execution_summary(
        &self,
        success_count: usize,
        failed_count: usize,
        failed_packages: &[String],
        command: &str,
    ) {
        Logger::info(format!(
            "\n{} {}",
            icons::SUMMARY,
            t!("run.execution_summary")
        ));
        Logger::info("═══════════════════════════════════════");

        Logger::info(tf!("run.summary_script", command));
        Logger::info(tf!("run.summary_success", success_count));
        Logger::info(tf!("run.summary_failed", failed_count));
        Logger::info(tf!("run.summary_total", success_count + failed_count));

        if failed_count > 0 {
            Logger::info(format!("\n{} {}", icons::ERROR, t!("run.failed_packages")));
            for package in failed_packages {
                Logger::error(format!("  {} {}", icons::PACKAGE, package));
            }
        }

        if failed_count == 0 {
            Logger::success(format!("{} {}", icons::SUCCESS, t!("run.all_success")));
        } else {
            Logger::warn(format!("{} {}", icons::WARNING, t!("run.partial_success")));
        }
    }
}
