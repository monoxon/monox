// ============================================================================
// MonoX - CLI Run 命令
// ============================================================================
//
// 文件: src/cli/run.rs
// 职责: 脚本运行命令的 CLI 接口层
// 边界:
//   - ✅ 命令行参数定义和解析
//   - ✅ 调用核心执行器运行脚本
//   - ❌ 不应包含具体脚本执行逻辑
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

use anyhow::Result;
use clap::Args;

use crate::core::TaskExecutor;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 运行脚本命令
#[derive(Debug, Args)]
pub struct RunArgs {
    /// 要执行的脚本命令 (如: build, dev, test) - must
    #[arg(short = 'c', long)]
    pub command: String,

    /// 目标包名列表 (如果不指定，将运行所有包含该脚本的包) - no must
    #[arg(short = 'p', long)]
    pub package_name: Option<String>,

    /// 是否运行所有包 - no must
    #[arg(short = 'a', long)]
    pub all: bool,
}

pub fn run(args: RunArgs) -> Result<()> {
    Logger::info(tf!("run.start", &args.command));

    let executor = TaskExecutor::new_from_config()?;
    match (args.all, args.package_name) {
        (true, _) => executor.execute("*", &args.command, Some(true)),
        (false, Some(package_name)) => executor.execute(&package_name, &args.command, Some(false)),
        (false, None) => anyhow::bail!(t!("run.missing_package_or_all")),
    }
}
