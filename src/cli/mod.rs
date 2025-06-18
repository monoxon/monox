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
pub mod fix;
pub mod init;
pub mod run;

use anyhow::Result;
use clap::{Parser, Subcommand};

use analyze::{handle_analyze, AnalyzeArgs};
use check::{handle_check, CheckArgs};
use fix::{handle_fix, FixArgs};
use init::{handle_init, InitArgs};
use run::{run, RunArgs};

/// MonoX - 轻量级 monorepo 构建工具
#[derive(Debug, Parser)]
#[command(name = "monox")]
#[command(about = "Lightweight monorepo build tool based on Rust")]
#[command(version)]
pub struct Cli {
    /// 全局详细模式
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// 界面语言 (zh_cn, en_us)
    #[arg(short, long, global = true)]
    pub language: Option<String>,

    /// 工作区根目录
    #[arg(short = 'C', long, global = true)]
    pub workspace_root: Option<String>,

    /// 最大并发数
    #[arg(short = 'j', long, global = true)]
    pub max_concurrency: Option<usize>,

    /// 任务超时时间（秒）
    #[arg(long, global = true)]
    pub timeout: Option<u32>,

    /// 重试次数
    #[arg(long, global = true)]
    pub retry: Option<u32>,

    /// 失败时继续执行
    #[arg(long, global = true)]
    pub continue_on_failure: bool,

    /// 禁用彩色输出
    #[arg(long, global = true)]
    pub no_color: bool,

    /// 禁用进度条
    #[arg(long, global = true)]
    pub no_progress: bool,

    /// 命令
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 分析工作区依赖关系
    Analyze(AnalyzeArgs),
    /// 检查工作区健康状态
    Check(CheckArgs),
    /// 自动修复版本冲突
    Fix(FixArgs),
    /// 初始化配置文件
    Init(InitArgs),
    /// 运行脚本
    Run(RunArgs),
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    // 构建运行时参数来覆盖配置
    let runtime_args = build_runtime_args(&cli);
    // 合并运行时参数到全局配置
    use crate::models::config::Config;
    Config::merge_runtime_args(runtime_args)?;

    match cli.command {
        Commands::Analyze(args) => handle_analyze(args),
        Commands::Check(args) => handle_check(args),
        Commands::Fix(args) => handle_fix(args),
        Commands::Init(args) => handle_init(args),
        Commands::Run(args) => run(args).await,
    }
}

/// 从 CLI 参数构建运行时参数
fn build_runtime_args(cli: &Cli) -> crate::models::config::RuntimeArgs {
    use crate::models::config::RuntimeArgs;

    RuntimeArgs {
        verbose: if cli.verbose { Some(true) } else { None },
        colored: if cli.no_color { Some(false) } else { None },
        show_progress: if cli.no_progress { Some(false) } else { None },
        max_concurrency: cli.max_concurrency,
        task_timeout: cli.timeout,
        retry_count: cli.retry,
        continue_on_failure: if cli.continue_on_failure {
            Some(true)
        } else {
            None
        },
        workspace_root: cli.workspace_root.clone(),
        language: cli.language.clone(),
    }
}
