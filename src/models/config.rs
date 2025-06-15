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
    pub package_manager: String,
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
        Self {
            workspace: WorkspaceConfig {
                root: ".".to_string(),
                package_manager: "pnpm".to_string(),
                ignore: vec![
                    ".git".to_string(),
                    "node_modules".to_string(),
                    "target".to_string(),
                    "dist".to_string(),
                    "build".to_string(),
                    ".next".to_string(),
                    ".nuxt".to_string(),
                    "coverage".to_string(),
                    "*.log".to_string(),
                    "tmp".to_string(),
                    "temp".to_string(),
                ],
            },
            tasks: vec![
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
            ],
            execution: ExecutionConfig {
                max_concurrency: num_cpus::get(),
                task_timeout: 300,
                retry_count: 1,
                continue_on_failure: false,
            },
            output: OutputConfig {
                show_progress: true,
                verbose: false,
                colored: true,
            },
            i18n: I18nConfig {
                language: "en_us".to_string(),
            },
        }
    }

    /// 生成默认配置模板并保存到文件
    pub fn create_default_config_file(config_path: &PathBuf) -> anyhow::Result<()> {
        let default_config = Self::generate_default_template();
        default_config.save_to_file(config_path)?;
        Ok(())
    }

    /// 获取工作区根目录
    pub fn get_workspace_root() -> anyhow::Result<PathBuf> {
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

    /// 获取最大并发数
    pub fn get_max_concurrency() -> anyhow::Result<usize> {
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

    /// 获取是否详细输出
    pub fn get_verbose() -> anyhow::Result<bool> {
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

    /// 获取包管理器类型
    pub fn get_package_manager() -> anyhow::Result<String> {
        let global_config = GLOBAL_CONFIG
            .get()
            .ok_or_else(|| anyhow::anyhow!("Global config not initialized"))?;

        let config = global_config
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire config read lock"))?;

        Ok(config.workspace.package_manager.clone())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace: WorkspaceConfig::default(),
            tasks: Vec::new(),
            execution: ExecutionConfig::default(),
            output: OutputConfig::default(),
            i18n: I18nConfig::default(),
        }
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            root: ".".to_string(),
            package_manager: "npm".to_string(),
            ignore: vec!["node_modules".to_string(), ".git".to_string()],
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 1,
            task_timeout: 60,
            retry_count: 0,
            continue_on_failure: false,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            show_progress: false,
            verbose: false,
            colored: false,
        }
    }
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            // 默认使用英文
            language: "en_us".to_string(),
        }
    }
}
