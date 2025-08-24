// ============================================================================
// MonoX - CLI 模块
// ============================================================================
//
// 文件: src/cli/mod.rs
// 职责: CLI 命令行接口模块入口和路由
// 边界:
//   - ✅ CLI 结构定义和命令枚举
//   - ✅ 命令行参数解析配置
//   - ✅ 命令路由分发
//   - ✅ 子模块导出
//   - ❌ 不应包含具体命令实现逻辑
//   - ❌ 不应包含业务逻辑处理
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

pub mod analyze;
pub mod check;
pub mod exec;
pub mod fix;
pub mod init;
pub mod run;
pub mod update;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::models::config::{Config, RuntimeArgs};
use analyze::{handle_analyze, AnalyzeArgs};
use check::{handle_check, CheckArgs};
use exec::{exec, ExecArgs};
use fix::{handle_fix, FixArgs};
use init::{handle_init, InitArgs};
use run::{run, RunArgs};
use update::{handle_update, UpdateArgs};

/// MonoX - Lightweight monorepo build tool
#[derive(Debug, Parser)]
#[command(name = "monox")]
#[command(about = "Lightweight monorepo build tool based on Rust")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Global verbose mode
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Interface language (zh_cn, en_us)
    #[arg(short, long, global = true)]
    pub language: Option<String>,

    /// Workspace root directory
    #[arg(short = 'C', long, global = true)]
    pub workspace_root: Option<String>,

    /// Maximum concurrency
    #[arg(short = 'j', long, global = true)]
    pub max_concurrency: Option<usize>,

    /// Task timeout (seconds)
    #[arg(long, global = true)]
    pub timeout: Option<u32>,

    /// Retry count
    #[arg(long, global = true)]
    pub retry: Option<u32>,

    /// Continue on failure
    #[arg(long, global = true)]
    pub continue_on_failure: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Disable progress bar
    #[arg(long, global = true)]
    pub no_progress: bool,

    /// Commands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Analyze workspace dependency relationships
    Analyze(AnalyzeArgs),
    /// Check workspace health status
    Check(CheckArgs),
    /// Execute predefined tasks
    Exec(ExecArgs),
    /// Auto-fix version conflicts
    Fix(FixArgs),
    /// Initialize configuration file
    Init(InitArgs),
    /// Run scripts
    Run(RunArgs),
    /// Update dependencies to latest versions
    Update(UpdateArgs),
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    // Build runtime args to override config
    let runtime_args = build_runtime_args(&cli);
    // Merge runtime args to global config
    Config::merge_runtime_args(runtime_args)?;

    match cli.command {
        Commands::Analyze(args) => handle_analyze(args),
        Commands::Check(args) => handle_check(args).await,
        Commands::Exec(args) => exec(args).await,
        Commands::Fix(args) => handle_fix(args),
        Commands::Init(args) => handle_init(args),
        Commands::Run(args) => run(args).await,
        Commands::Update(args) => handle_update(args).await,
    }
}

/// Build runtime args from CLI arguments
fn build_runtime_args(cli: &Cli) -> crate::models::config::RuntimeArgs {
    RuntimeArgs {
        verbose: if cli.verbose { Some(true) } else { None },
        colored: if cli.no_color { Some(false) } else { None },
        show_progress: if cli.no_progress { Some(false) } else { None },
        max_concurrency: cli.max_concurrency,
        task_timeout: cli.timeout,
        retry_count: cli.retry,
        continue_on_failure: if cli.continue_on_failure { Some(true) } else { None },
        workspace_root: cli.workspace_root.clone(),
        language: cli.language.clone(),
    }
}
