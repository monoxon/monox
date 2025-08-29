// ============================================================================
// MonoX - 任务数据模型
// ============================================================================
//
// 文件: src/models/task.rs
// 职责: 任务执行相关的数据结构定义
// 边界:
//   - 任务信息数据结构定义
//   - 任务状态枚举定义
//   - 执行结果数据结构定义
//   - 任务配置数据结构定义
//   - 不应包含任务执行逻辑
//   - 不应包含任务调度逻辑
//   - 不应包含 CLI 相关逻辑
//   - 不应包含文件操作逻辑
//
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

use crate::models::config::Config;
use crate::models::package::PackageJson;

/// 任务状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
    /// 已跳过
    Skipped,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Running => write!(f, "Running"),
            TaskStatus::Success => write!(f, "Success"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Skipped => write!(f, "Skipped"),
        }
    }
}

/// 任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务ID
    pub id: String,
    /// 包名
    pub package_name: String,
    /// 包路径
    pub package_path: String,
    /// 执行命令
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 工作目录
    pub working_directory: String,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 任务状态
    pub status: TaskStatus,
    /// 创建时间
    pub created_at: SystemTime,
    /// 开始时间
    pub started_at: Option<SystemTime>,
    /// 完成时间
    pub completed_at: Option<SystemTime>,
    /// 执行结果
    pub result: Option<TaskResult>,
}

/// 任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 退出状态码
    pub exit_code: i32,
    /// 标准输出
    pub stdout: String,
    /// 标准错误输出
    pub stderr: String,
    /// 执行时长
    pub duration: Duration,
    /// 是否成功
    pub success: bool,
}

/// 任务执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// 最大并发数
    pub max_concurrency: usize,
    /// 任务超时时间（秒）
    pub timeout_seconds: Option<u64>,
    /// 失败重试次数
    pub retry_count: u32,
    /// 是否继续执行其他任务（当某个任务失败时）
    pub continue_on_error: bool,
    /// 是否静默模式
    pub silent: bool,
    /// 是否显示详细输出
    pub verbose: bool,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            max_concurrency: num_cpus::get(),
            timeout_seconds: None,
            retry_count: 0,
            continue_on_error: false,
            silent: false,
            verbose: false,
        }
    }
}

impl Task {
    /// 创建新任务
    pub fn new(
        package_name: String,
        package_path: String,
        command: String,
        args: Vec<String>,
    ) -> Self {
        let id = format!("{}:{}", package_name, command);
        Self {
            id,
            package_name: package_name.clone(),
            package_path: package_path.clone(),
            command,
            args,
            working_directory: package_path,
            env_vars: HashMap::new(),
            status: TaskStatus::Pending,
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            result: None,
        }
    }

    /// 设置环境变量
    pub fn with_env_vars(mut self, env_vars: HashMap<String, String>) -> Self {
        self.env_vars = env_vars;
        self
    }

    /// 设置工作目录
    pub fn with_working_directory(mut self, working_directory: String) -> Self {
        self.working_directory = working_directory;
        self
    }

    /// 开始执行
    pub fn start(&mut self) {
        match self.has_script(self.command.as_str()) {
            true => {
                self.status = TaskStatus::Running;
                self.started_at = Some(SystemTime::now());
            }
            false => {
                self.skip();
            }
        }
    }

    /// 完成执行
    pub fn complete(&mut self, result: TaskResult) {
        self.status = if result.success { TaskStatus::Success } else { TaskStatus::Failed };
        self.completed_at = Some(SystemTime::now());
        self.result = Some(result);
    }

    pub fn has_script(&self, script_name: &str) -> bool {
        let workspace_root = Config::get_workspace_root();

        let package_json = PackageJson::from_file(
            &workspace_root.to_path_buf().join(self.package_path.as_str()).to_string_lossy(),
        );
        package_json.has_script(script_name)
    }

    /// 跳过执行
    pub fn skip(&mut self) {
        self.status = TaskStatus::Skipped;
        self.completed_at = Some(SystemTime::now());
    }

    /// 获取执行时长
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => end.duration_since(start).ok(),
            _ => None,
        }
    }

    /// 判断是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Success | TaskStatus::Failed | TaskStatus::Skipped)
    }

    /// 判断是否成功
    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Success
    }

    /// 判断是否失败
    pub fn is_failed(&self) -> bool {
        self.status == TaskStatus::Failed
    }
}

impl TaskResult {
    /// 创建成功结果
    pub fn success(stdout: String, duration: Duration) -> Self {
        Self { exit_code: 0, stdout, stderr: String::new(), duration, success: true }
    }

    /// 创建失败结果
    pub fn failure(exit_code: i32, stdout: String, stderr: String, duration: Duration) -> Self {
        Self { exit_code, stdout, stderr, duration, success: false }
    }
}
