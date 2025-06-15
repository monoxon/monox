// ============================================================================
// MonoX - 初始化命令处理
// ============================================================================
//
// 文件: src/cli/init.rs
// 职责: 处理配置文件初始化命令
// 边界:
//   - ✅ 初始化命令参数解析
//   - ✅ 默认配置文件生成
//   - ✅ 配置文件存在性检查
//   - ✅ 用户交互和确认
//   - ❌ 不应包含配置文件格式定义
//   - ❌ 不应包含业务逻辑处理
//   - ❌ 不应包含文件系统底层操作
//   - ❌ 不应包含配置验证逻辑
//
// ============================================================================

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::models::config::Config;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 初始化命令参数
#[derive(Debug, Args)]
pub struct InitArgs {
    /// 配置文件路径
    #[arg(short, long, default_value = "monox.toml")]
    pub config: PathBuf,

    /// 强制覆盖已存在的配置文件
    #[arg(short, long)]
    pub force: bool,
}

/// 处理初始化命令
pub fn handle_init(args: InitArgs) -> Result<()> {
    Logger::info(t!("init.start"));

    // 检查配置文件是否已存在
    if args.config.exists() && !args.force {
        Logger::warn(tf!("init.config_exists", args.config.display()));
        Logger::info(t!("init.use_force_hint"));
        return Ok(());
    }

    // 生成默认配置文件
    match Config::create_default_config_file(&args.config) {
        Ok(_) => {
            Logger::info(tf!("init.config_created", args.config.display()));
            Logger::info(t!("init.next_steps"));
        }
        Err(e) => {
            Logger::error(tf!("init.create_failed", e));
            return Err(e);
        }
    }

    Ok(())
}
