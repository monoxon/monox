// ============================================================================
// MonoX - CLI Exec 命令
// ============================================================================
//
// 文件: src/cli/exec.rs
// 职责: 命令执行命令的 CLI 接口层
// 边界:
//   - ✅ 命令行参数定义和解析
//   - ✅ 调用核心执行器执行命令
//   - ✅ 执行结果格式化输出
//   - ✅ 用户交互和提示信息
//   - ❌ 不应包含具体命令执行逻辑
//   - ❌ 不应包含进程管理逻辑
//   - ❌ 不应包含并发控制逻辑
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

use anyhow::Result;
use clap::Args;

use crate::core::TaskExecutor;
use crate::models::config::Config;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 执行预定义任务命令
#[derive(Debug, Args)]
pub struct ExecArgs {
    /// 要执行的任务名称（在 monox.toml 中定义）
    #[arg(short = 't', long)]
    pub task: String,
}

/// 执行预定义任务
pub async fn exec(args: ExecArgs) -> Result<()> {
    Logger::info(tf!("exec.start", &args.task));

    // 从配置文件中获取任务定义
    let task_config = Config::get_task_config(&args.task)
        .map_err(|_| anyhow::anyhow!(tf!("exec.task_not_found", &args.task)))?;

    Logger::info(tf!("exec.task_found", &task_config.name, &task_config.command));

    if let Some(desc) = &task_config.desc {
        Logger::info(tf!("exec.task_description", desc));
    }

    // 创建任务执行器
    let executor = TaskExecutor::new_from_config()?;

    // 根据配置决定执行策略
    if let Some(packages) = &task_config.packages {
        // 如果配置了 packages 字段，执行多包
        if packages.is_empty() {
            anyhow::bail!(t!("exec.empty_packages_list"));
        }
        Logger::info(tf!("exec.executing_packages", packages.join(", ")));
        executor.execute_packages(packages, &task_config.command, &task_config.post_command).await
    } else if !task_config.pkg_name.is_empty() {
        // 如果有 pkg_name 且不为空，按原逻辑处理
        let is_all_packages = task_config.pkg_name == "*";

        if is_all_packages {
            Logger::info(t!("exec.executing_all_packages"));
            executor.execute("*", &task_config.command, &task_config.post_command, Some(true)).await
        } else {
            Logger::info(tf!("exec.executing_package", &task_config.pkg_name));
            executor
                .execute(
                    &task_config.pkg_name,
                    &task_config.command,
                    &task_config.post_command,
                    Some(false),
                )
                .await
        }
    } else {
        // 如果既没有 packages 也没有 pkg_name，报错
        anyhow::bail!(t!("exec.missing_target_config"));
    }
}
