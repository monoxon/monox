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

use crate::core::DependencyAnalyzer;
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
use std::time::Instant;

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
    pub fn execute(&self, package_name: &str, command: &str, all: Option<bool>) -> Result<()> {
        match (all.unwrap_or(false), package_name) {
            // all 为 true 时，执行所有包
            (true, _) => self.execute_all_packages(command),
            // all 为 false，且有 package_name 时，执行单包
            (false, pkg_name) => self.execute_single_package(pkg_name, command),
        }
    }

    /// 执行所有包（all = true）
    fn execute_all_packages(&self, command: &str) -> Result<()> {
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

        self.execute_stages(&analysis_result.stages, command)
    }

    /// 执行单个包
    fn execute_single_package(&self, package_name: &str, command: &str) -> Result<()> {
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

        self.execute_stages(&analysis_result.stages, command)
    }

    /// 执行阶段任务
    fn execute_stages(&self, stages: &Vec<Vec<WorkspacePackage>>, command: &str) -> Result<()> {
        for stage in stages {
            self.execute_single_stage(stage, command)?;
        }
        Ok(())
    }

    /// 单个阶段任务
    fn execute_single_stage(&self, stage: &Vec<WorkspacePackage>, command: &str) -> Result<()> {
        for package in stage {
            let mut task = Task::new(
                package.name.clone(),
                package.folder.to_string_lossy().to_string(),
                command.to_string(),
                vec![],
            );
            self.execute_task(&mut task)?; // 串行执行，遇到错误立即停止
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

    /// 执行单个任务
    pub fn execute_task(&self, task: &mut Task) -> Result<()> {
        if self.config.verbose {
            Logger::info(tf!(
                "executor.task_start",
                &task.package_name,
                &task.command
            ));
        }

        // 开始执行
        task.start();

        if task.status == TaskStatus::Skipped {
            return Ok(());
        }

        let start_time = Instant::now();

        // 执行命令
        let result = self.run_command(task)?;

        // 更新任务状态
        task.complete(result);

        // 输出结果
        if task.is_success() {
            if self.config.verbose {
                Logger::success(tf!(
                    "executor.task_success",
                    &task.package_name,
                    &task.command,
                    start_time.elapsed().as_secs_f64()
                ));
            }
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

    /// 执行命令并返回结果
    fn run_command(&self, task: &Task) -> Result<TaskResult> {
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

        if self.config.verbose {
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
        let current_path = String::from_utf8_lossy(&output.stdout);
        Logger::info(format!("当前工作目录: {}", current_path.trim()));

        let duration = start_time.elapsed();
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        // 在详细模式下输出命令输出
        if self.config.verbose {
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

    /// 检查脚本是否在 package.json 中可用
    pub fn check_script_available(&self, package_dir: &str, script_name: &str) -> bool {
        let package_json_path = Path::new(package_dir).join("package.json");

        if !package_json_path.exists() {
            return false;
        }

        match fs::read_to_string(&package_json_path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    if let Some(scripts) = json.get("scripts") {
                        if let Some(scripts_obj) = scripts.as_object() {
                            return scripts_obj.contains_key(script_name);
                        }
                    }
                    false
                }
                Err(_) => false,
            },
            Err(_) => false,
        }
    }

    /// 检查命令是否可执行（保留原有功能，用于检查系统命令）
    pub fn check_command_available(&self, command: &str) -> bool {
        let output = Command::new("which").arg(command).output().or_else(|_| {
            // 在 Windows 上使用 where 命令
            Command::new("where").arg(command).output()
        });

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}
