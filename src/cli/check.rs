// ============================================================================
// MonoX - CLI Check 命令
// ============================================================================
//
// 文件: src/cli/check.rs
// 职责: 代码检查命令的 CLI 接口层
// 边界:
//   - ✅ 命令行参数定义和解析
//   - ✅ 调用核心检查器执行检查
//   - ✅ 检查结果格式化输出
//   - ✅ 用户交互和提示信息
//   - ✅ 进度显示和用户反馈
//   - ❌ 不应包含具体检查逻辑
//   - ❌ 不应包含文件扫描逻辑
//   - ❌ 不应包含规则定义
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

use anyhow::Result;
use clap::Args;
use std::sync::{Arc, Mutex};

use crate::core::checker::{HealthChecker, OutdatedDependency, ProgressCallback};
use crate::models::config::Config;
use crate::ui::spinner::Spinner;
use crate::ui::summary;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 检查工作区健康状态
#[derive(Debug, Args)]
pub struct CheckArgs {
    /// 检查循环依赖
    #[arg(long)]
    pub circular: bool,

    /// 检查版本冲突
    #[arg(long)]
    pub versions: bool,

    /// 检查过期依赖
    #[arg(long)]
    pub outdated: bool,

    /// 输出格式 (table, json)
    #[arg(short = 'f', long, default_value = "table")]
    pub format: String,

    /// 显示详细信息
    #[arg(short = 'd', long)]
    pub detail: bool,
}

pub async fn handle_check(args: CheckArgs) -> Result<()> {
    Logger::info(t!("cli.check.start"));

    let workspace_root = Config::get_workspace_root();
    let verbose = Config::get_verbose();

    if !workspace_root.exists() {
        anyhow::bail!(tf!("error.workspace_not_exist", workspace_root.display()));
    }

    // 创建健康检查器
    let checker = HealthChecker::new(workspace_root.clone()).with_verbose(verbose);

    // 确定检查项目
    let check_items = determine_check_items(&args);
    let mut has_issues = false;

    // 执行各项检查
    if check_items.circular {
        has_issues |= check_circular_dependencies(&checker, verbose, &args)?;
    }
    if check_items.versions {
        has_issues |= check_version_conflicts(&checker, verbose, &args)?;
    }
    if check_items.outdated {
        has_issues |= check_outdated_dependencies(&checker, verbose, &args).await?;
    }

    // 输出结果
    if has_issues {
        std::process::exit(1);
    } else {
        Logger::success(t!("check.all_good"));
    }

    Ok(())
}

/// 检查项目配置
struct CheckItems {
    circular: bool,
    versions: bool,
    outdated: bool,
}

/// 确定要执行的检查项目
fn determine_check_items(args: &CheckArgs) -> CheckItems {
    CheckItems {
        circular: args.circular || (!args.versions && !args.outdated),
        versions: args.versions,
        outdated: args.outdated,
    }
}

/// 检查循环依赖
fn check_circular_dependencies(
    checker: &HealthChecker,
    verbose: bool,
    args: &CheckArgs,
) -> Result<bool> {
    if verbose {
        Logger::info(t!("check.circular.start"));
    }

    let circular_dependencies = checker.check_circular_dependencies()?;

    if circular_dependencies.is_empty() {
        Logger::success(t!("check.circular.none_found"));
        return Ok(false);
    }

    Logger::error(tf!("check.circular.found", circular_dependencies.len()));

    output_results(
        &args.format,
        &circular_dependencies,
        args.detail,
        |deps, detail| summary::print_circular_dependencies_table(deps, detail),
    )?;

    Ok(true)
}

/// 检查版本冲突
fn check_version_conflicts(
    checker: &HealthChecker,
    verbose: bool,
    args: &CheckArgs,
) -> Result<bool> {
    if verbose {
        Logger::info(t!("check.versions.start"));
    }

    let version_conflicts = checker.check_version_conflicts()?;
    if version_conflicts.is_empty() {
        Logger::success(t!("check.versions.none_found"));
        return Ok(false);
    }

    Logger::error(tf!("check.versions.found", version_conflicts.len()));

    // 转换为 summary 模块的类型
    let summary_conflicts: Vec<summary::VersionConflict> = version_conflicts
        .into_iter()
        .map(|c| summary::VersionConflict {
            name: c.name,
            conflicts: c
                .conflicts
                .into_iter()
                .map(|usage| summary::ConflictUsage {
                    package: usage.package,
                    version_spec: usage.version_spec,
                    resolved_version: usage.resolved_version,
                    dep_type: usage.dep_type,
                })
                .collect(),
            recommended_version: c.recommended_version,
        })
        .collect();

    output_results(
        &args.format,
        &summary_conflicts,
        args.detail,
        |conflicts, detail| summary::print_version_conflicts_table(conflicts, detail),
    )?;

    Ok(true)
}

/// 检查过期依赖
async fn check_outdated_dependencies(
    checker: &HealthChecker,
    verbose: bool,
    args: &CheckArgs,
) -> Result<bool> {
    if verbose {
        Logger::info(t!("check.outdated.start"));
    }

    // 创建进度显示和回调
    let spinner = if !verbose {
        let mut s = Spinner::new_with_prefix(
            Logger::get_prefix("INFO"),
            tf!("check.outdated.progress", 0, 0),
        );
        s.start();
        Some(Arc::new(Mutex::new(s)))
    } else {
        None
    };

    // 创建进度回调
    let progress_callback: Option<ProgressCallback> = if let Some(ref spinner_clone) = spinner {
        let spinner_for_callback = Arc::clone(spinner_clone);
        Some(Arc::new(move |completed: usize, total: usize| {
            if verbose {
                Logger::info(tf!("check.outdated.progress", completed, total));
            } else if let Ok(s) = spinner_for_callback.lock() {
                s.update_message(tf!("check.outdated.progress", completed, total));
            }
        }))
    } else if verbose {
        Some(Arc::new(move |completed: usize, total: usize| {
            Logger::info(tf!("check.outdated.progress", completed, total));
        }))
    } else {
        None
    };

    // 执行检查
    let (result, total_checked) = checker
        .check_outdated_dependencies_with_progress(progress_callback)
        .await?;

    // 停止进度显示
    if let Some(spinner_arc) = spinner {
        if let Ok(mut s) = spinner_arc.lock() {
            s.stop();
        }
    }

    if result.is_empty() {
        // 即使没有过期依赖，也要显示统计信息
        log_outdated_found_message_with_total(total_checked, 0, 0);
        return Ok(false);
    }

    let unique_outdated_count = get_unique_outdated_count(&result);

    // 转换为 summary 模块的类型
    let summary_outdated: Vec<summary::OutdatedDependency> = result
        .into_iter()
        .map(|dep| summary::OutdatedDependency {
            name: dep.name,
            current: dep.current,
            latest: dep.latest,
            package: dep.package,
            dep_type: dep.dep_type,
        })
        .collect();

    output_results(
        &args.format,
        &summary_outdated,
        args.detail,
        |deps, detail| summary::print_outdated_dependencies_table(deps, detail),
    )?;

    // 在汇总表格之后打印数量信息，确保用户能看到
    log_outdated_found_message_with_total(
        total_checked,
        unique_outdated_count,
        summary_outdated.len(),
    );

    Ok(true)
}

/// 通用结果输出函数
fn output_results<T, F>(format: &str, data: &T, detail: bool, print_table: F) -> Result<()>
where
    T: serde::Serialize + ?Sized,
    F: FnOnce(&T, bool) -> Result<()>,
{
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        "table" | _ => {
            print_table(data, detail)?;
        }
    }
    Ok(())
}

/// 获取唯一过期包数量
fn get_unique_outdated_count(result: &[OutdatedDependency]) -> usize {
    result
        .iter()
        .map(|dep| &dep.name)
        .collect::<std::collections::HashSet<_>>()
        .len()
}

/// 记录过期依赖检查结果（包含总检测数量）
fn log_outdated_found_message_with_total(
    total_checked: usize,
    unique_count: usize,
    instance_count: usize,
) {
    if unique_count == 0 {
        // 未发现过期依赖，使用成功提示
        Logger::success(tf!("check.outdated.summary_clean", total_checked));
    } else if unique_count == instance_count {
        // 发现过期依赖，没有重复引用的情况，使用错误提示
        Logger::error(tf!(
            "check.outdated.found_with_total",
            total_checked,
            unique_count
        ));
    } else {
        // 发现过期依赖，有重复引用的情况，使用错误提示
        Logger::error(tf!(
            "check.outdated.found_with_total_and_instances",
            total_checked,
            unique_count,
            instance_count
        ));
    }
}
