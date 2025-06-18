// ============================================================================
// MonoX - 配置数据模型
// ============================================================================
//
// 文件: src/models/config.rs
// 职责: 配置文件数据结构定义和操作
// 边界:
//   - ✅ 配置文件数据结构定义
//   - ✅ 配置序列化/反序列化
//   - ✅ 配置验证和默认值
//   - ✅ 配置文件读写操作
//   - ✅ 配置项默认数据
//   - ❌ 不应包含配置应用逻辑
//   - ❌ 不应包含业务规则验证
//   - ❌ 不应包含 CLI 参数处理
//   - ❌ 不应包含文件系统底层操作
//
// ============================================================================

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 全局配置管理器
static GLOBAL_CONFIG: std::sync::OnceLock<Arc<RwLock<Config>>> = std::sync::OnceLock::new();

/// MonoX 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 工作空间配置
    #[serde(default)]
    pub workspace: WorkspaceConfig,
    /// 任务定义
    #[serde(default)]
    pub tasks: Vec<TaskConfig>,
    /// 执行配置
    #[serde(default)]
    pub execution: ExecutionConfig,
    /// 输出配置
    #[serde(default)]
    pub output: OutputConfig,
    /// 国际化配置
    #[serde(default)]
    pub i18n: I18nConfig,
}

/// 工作空间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// 工作区根目录
    #[serde(default)]
    pub root: String,
    /// 包管理器类型
    #[serde(default)]
    pub package_manager: PackageManager,
    /// 排除扫描的目录或文件模式
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// 任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// 任务名称
    pub name: String,
    /// 包名（"*" 表示所有包）
    pub pkg_name: String,
    /// 任务描述
    #[serde(default)]
    pub desc: Option<String>,
    /// 执行的命令
    pub command: String,
}

/// 执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// 最大并发数
    #[serde(default)]
    pub max_concurrency: usize,
    /// 任务超时时间（秒）
    #[serde(default)]
    pub task_timeout: u32,
    /// 重试次数
    #[serde(default)]
    pub retry_count: u32,
    /// 失败时是否继续
    #[serde(default)]
    pub continue_on_failure: bool,
}

/// 输出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 是否显示进度条
    #[serde(default)]
    pub show_progress: bool,
    /// 是否详细输出
    #[serde(default)]
    pub verbose: bool,
    /// 是否彩色输出
    #[serde(default)]
    pub colored: bool,
}

/// 国际化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    /// 界面语言
    #[serde(default)]
    pub language: String,
}

/// CLI 运行时参数（用于覆盖配置文件）
#[derive(Debug, Clone, Default)]
pub struct RuntimeArgs {
    pub verbose: Option<bool>,
    pub colored: Option<bool>,
    pub show_progress: Option<bool>,
    pub max_concurrency: Option<usize>,
    pub task_timeout: Option<u32>,
    pub retry_count: Option<u32>,
    pub continue_on_failure: Option<bool>,
    pub workspace_root: Option<String>,
    pub language: Option<String>,
}

/// 包管理器类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    /// pnpm 包管理器
    Pnpm,
    /// yarn 包管理器  
    Yarn,
    /// npm 包管理器
    Npm,
}

impl PackageManager {
    /// 获取包管理器命令字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageManager::Pnpm => "pnpm",
            PackageManager::Yarn => "yarn",
            PackageManager::Npm => "npm",
        }
    }

    /// 从字符串解析包管理器
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "pnpm" => Ok(PackageManager::Pnpm),
            "yarn" => Ok(PackageManager::Yarn),
            "npm" => Ok(PackageManager::Npm),
            _ => Err(format!("不支持的包管理器: {}，仅支持 pnpm、yarn、npm", s)),
        }
    }

    /// 获取所有支持的包管理器
    pub fn all() -> &'static [PackageManager] {
        &[
            PackageManager::Pnpm,
            PackageManager::Yarn,
            PackageManager::Npm,
        ]
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        PackageManager::Pnpm
    }
}

impl std::fmt::Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 配置默认值 trait - 不依赖全局配置初始化
pub trait ConfigDefaults {
    /// 获取默认工作区根目录
    fn default_workspace_root() -> PathBuf {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    /// 获取默认包管理器
    fn default_package_manager() -> PackageManager {
        PackageManager::Pnpm
    }

    /// 获取默认忽略模式
    fn default_ignore_patterns() -> Vec<String> {
        vec![".git".to_string(), "dist".to_string(), "*.log".to_string()]
    }

    /// 获取默认最大并发数
    fn default_max_concurrency() -> usize {
        num_cpus::get()
    }

    /// 获取默认任务超时时间
    fn default_task_timeout() -> u32 {
        300
    }

    /// 获取默认重试次数
    fn default_retry_count() -> u32 {
        1
    }

    /// 获取默认是否失败时继续
    fn default_continue_on_failure() -> bool {
        false
    }

    /// 获取默认是否显示进度条
    fn default_show_progress() -> bool {
        true
    }

    /// 获取默认是否详细输出
    fn default_verbose() -> bool {
        false
    }

    /// 获取默认是否彩色输出
    fn default_colored() -> bool {
        true
    }

    /// 获取默认语言
    fn default_language() -> String {
        "en_us".to_string()
    }
}

impl ConfigDefaults for Config {}

impl Config {
    /// 初始化全局配置（程序启动时调用）
    pub fn initialize() -> anyhow::Result<()> {
        let config = Self::load_config()?;
        GLOBAL_CONFIG
            .set(Arc::new(RwLock::new(config)))
            .map_err(|_| anyhow::anyhow!("Global config already initialized"))?;
        Ok(())
    }

    /// 加载配置文件
    fn load_config() -> anyhow::Result<Self> {
        let config_path = PathBuf::from("monox.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // 如果配置文件不存在，使用默认配置
            Ok(Self::default())
        }
    }

    /// 合并运行时参数
    pub fn merge_runtime_args(args: RuntimeArgs) -> anyhow::Result<()> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let mut config = global_config
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config write lock"))?;

        // 合并参数
        if let Some(verbose) = args.verbose {
            config.output.verbose = verbose;
        }
        if let Some(colored) = args.colored {
            config.output.colored = colored;
        }
        if let Some(show_progress) = args.show_progress {
            config.output.show_progress = show_progress;
        }
        if let Some(max_concurrency) = args.max_concurrency {
            config.execution.max_concurrency = max_concurrency;
        }
        if let Some(task_timeout) = args.task_timeout {
            config.execution.task_timeout = task_timeout;
        }
        if let Some(retry_count) = args.retry_count {
            config.execution.retry_count = retry_count;
        }
        if let Some(continue_on_failure) = args.continue_on_failure {
            config.execution.continue_on_failure = continue_on_failure;
        }
        if let Some(workspace_root) = args.workspace_root {
            config.workspace.root = workspace_root;
        }
        if let Some(language) = args.language {
            config.i18n.language = language;
        }

        Ok(())
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, config_path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// 生成默认配置模板
    pub fn generate_default_template() -> Self {
        let mut config = Self::default();

        // 添加示例任务
        config.tasks = vec![
            TaskConfig {
                name: "build".to_string(),
                pkg_name: "*".to_string(),
                desc: Some("构建所有包".to_string()),
                command: "npm run build".to_string(),
            },
            TaskConfig {
                name: "test".to_string(),
                pkg_name: "*".to_string(),
                desc: Some("运行测试".to_string()),
                command: "npm run test".to_string(),
            },
            TaskConfig {
                name: "lint".to_string(),
                pkg_name: "*".to_string(),
                desc: Some("代码检查".to_string()),
                command: "npm run lint".to_string(),
            },
        ];

        config
    }

    /// 生成默认配置模板并保存到文件
    pub fn create_default_config_file(config_path: &PathBuf) -> anyhow::Result<()> {
        let default_config = Self::generate_default_template();
        default_config.save_to_file(config_path)?;
        Ok(())
    }

    /// 获取工作区根目录（带默认值）
    pub fn get_workspace_root() -> PathBuf {
        match Self::get_workspace_root_from_config() {
            Ok(root) => root,
            _ => Self::default_workspace_root(),
        }
    }

    /// 从配置获取工作区根目录（可能失败）
    fn get_workspace_root_from_config() -> anyhow::Result<PathBuf> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        let root = &config.workspace.root;
        if root == "." {
            Ok(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        } else {
            Ok(PathBuf::from(root))
        }
    }

    /// 获取忽略模式列表
    pub fn get_ignore_patterns() -> anyhow::Result<Vec<String>> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.workspace.ignore.clone())
    }

    /// 检查路径是否应该被忽略
    pub fn should_ignore_path(path: &str) -> anyhow::Result<bool> {
        // node_modules 始终被忽略
        if path.contains("node_modules") {
            return Ok(true);
        }

        let ignore_patterns = Self::get_ignore_patterns()?;

        // 检查用户配置的忽略模式
        for pattern in &ignore_patterns {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                // 直接匹配
                if glob_pattern.matches(path) {
                    return Ok(true);
                }
                // 也检查路径的开头部分是否匹配模式
                if path.starts_with(pattern) {
                    return Ok(true);
                }
                // 检查路径中是否包含该模式
                if path.contains(pattern) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// 获取界面语言
    pub fn get_language() -> anyhow::Result<String> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.i18n.language.clone())
    }

    /// 获取最大并发数（带默认值）
    pub fn get_max_concurrency() -> usize {
        match Self::get_max_concurrency_from_config() {
            Ok(concurrency) => concurrency,
            _ => Self::default_max_concurrency(),
        }
    }

    /// 从配置获取最大并发数（可能失败）
    fn get_max_concurrency_from_config() -> anyhow::Result<usize> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.execution.max_concurrency)
    }

    /// 获取任务超时时间
    pub fn get_task_timeout() -> anyhow::Result<u32> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.execution.task_timeout)
    }

    /// 获取重试次数
    pub fn get_retry_count() -> anyhow::Result<u32> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.execution.retry_count)
    }

    /// 获取失败时是否继续执行
    pub fn get_continue_on_failure() -> anyhow::Result<bool> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.execution.continue_on_failure)
    }

    /// 获取是否显示进度条
    pub fn get_show_progress() -> anyhow::Result<bool> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.output.show_progress)
    }

    /// 获取详细输出设置（带默认值）
    pub fn get_verbose() -> bool {
        match Self::get_verbose_from_config() {
            Ok(verbose) => verbose,
            _ => Self::default_verbose(),
        }
    }

    /// 从配置获取详细输出设置（可能失败）
    fn get_verbose_from_config() -> anyhow::Result<bool> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.output.verbose)
    }

    /// 获取是否彩色输出
    pub fn get_colored() -> anyhow::Result<bool> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.output.colored)
    }

    /// 获取包管理器类型（带默认值）
    pub fn get_package_manager() -> PackageManager {
        match Self::get_package_manager_from_config() {
            Ok(pm) => pm,
            _ => Self::default_package_manager(),
        }
    }

    /// 从配置获取包管理器（可能失败）
    fn get_package_manager_from_config() -> anyhow::Result<PackageManager> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.workspace.package_manager.clone())
    }

    /// 获取任务配置
    pub fn get_task_config(task_name: &str) -> anyhow::Result<TaskConfig> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        config
            .tasks
            .iter()
            .find(|task| task.name == task_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", task_name))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace: WorkspaceConfig {
                root: ".".to_string(),
                package_manager: Self::default_package_manager(),
                ignore: Self::default_ignore_patterns(),
            },
            tasks: Vec::new(),
            execution: ExecutionConfig {
                max_concurrency: Self::default_max_concurrency(),
                task_timeout: Self::default_task_timeout(),
                retry_count: Self::default_retry_count(),
                continue_on_failure: Self::default_continue_on_failure(),
            },
            output: OutputConfig {
                show_progress: Self::default_show_progress(),
                verbose: Self::default_verbose(),
                colored: Self::default_colored(),
            },
            i18n: I18nConfig {
                language: Self::default_language(),
            },
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            root: ".".to_string(),
            package_manager: Config::default_package_manager(),
            ignore: Config::default_ignore_patterns(),
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: Config::default_max_concurrency(),
            task_timeout: Config::default_task_timeout(),
            retry_count: Config::default_retry_count(),
            continue_on_failure: Config::default_continue_on_failure(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            show_progress: Config::default_show_progress(),
            verbose: Config::default_verbose(),
            colored: Config::default_colored(),
        }
    }
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            language: Config::default_language(),
        }
    }
}
