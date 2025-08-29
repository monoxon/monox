// ============================================================================
// MonoX - 执行结果汇总组件
// ============================================================================
//
// 文件: src/ui/summary.rs
// 职责: 执行结果汇总显示
// 边界:
//   - ✅ 执行结果汇总显示
//   - ✅ 统计信息格式化输出
//   - ✅ 国际化文本支持
//   - ✅ 检查结果表格显示
//   - ❌ 不应包含具体业务逻辑
//   - ❌ 不应包含任务执行逻辑
//   - ❌ 不应包含文件操作
//   - ❌ 不应包含数据处理逻辑
//
// ============================================================================

use anyhow::Result;
use std::collections::{BTreeMap, HashMap};
use std::io::{self, Write};

use crate::models::config::Config;
use crate::utils::colors::Colors;
use crate::utils::constants::icons;
use crate::utils::logger::Logger;
use crate::utils::styles::TextStyles;
use crate::{t, tf};

// ============================================================================
// 数据模型 (重新导入以避免循环依赖)
// ============================================================================

/// 过期依赖信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct OutdatedDependency {
    /// 包名
    pub name: String,
    /// 当前版本
    pub current: String,
    /// 最新版本
    pub latest: String,
    /// 所属包
    pub package: String,
    /// 依赖类型 (dependencies, devDependencies, etc.)
    pub dep_type: String,
}

/// 版本冲突信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct VersionConflict {
    /// 依赖包名
    pub name: String,
    /// 冲突的版本使用情况
    pub conflicts: Vec<ConflictUsage>,
    /// 推荐的统一版本
    pub recommended_version: String,
}

/// 版本冲突使用情况
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConflictUsage {
    /// 使用该版本的包名
    pub package: String,
    /// 版本规范
    pub version_spec: String,
    /// 解析后的版本
    pub resolved_version: String,
    /// 依赖类型
    pub dep_type: String,
}

// ============================================================================
// 执行汇总显示
// ============================================================================

/// 渲染执行汇总
pub fn render_execution_summary(
    total_tasks: usize,
    successful_tasks: usize,
    failed_tasks: usize,
    skipped_tasks: usize,
    duration_ms: Option<u64>,
) {
    // 构建汇总内容
    let mut summary_lines = vec![
        "".to_string(),
        TextStyles::bold(&t!("runner.execution_summary")),
        "═══════════════════════════════════════".to_string(),
        format!("{} {}", icons::PACKAGE, tf!("runner.total_tasks", total_tasks)),
        format!("{} {}", icons::SUCCESS, tf!("runner.successful_tasks", successful_tasks)),
        format!("{} {}", icons::ERROR, tf!("runner.failed_tasks", failed_tasks)),
        format!("{} {}", icons::SKIP, tf!("runner.skipped_tasks", skipped_tasks)),
    ];

    // 如果有执行时长信息，添加到汇总中
    if let Some(duration) = duration_ms {
        summary_lines.push(format!(
            "{} {}",
            icons::TIME,
            tf!("executor.summary_duration", duration as f64 / 1000.0)
        ));
    }

    // 输出汇总内容
    for line in summary_lines {
        Logger::info(line);
    }

    let _ = io::stdout().flush();
}

// ============================================================================
// 检查结果汇总显示
// ============================================================================

/// 打印循环依赖表格
pub fn print_circular_dependencies_table(
    circular_dependencies: &[Vec<String>],
    detail: bool,
) -> Result<()> {
    Logger::info("");
    Logger::info(t!("check.circular.details"));
    Logger::info("───────────────────────────────────────");

    for (index, cycle) in circular_dependencies.iter().enumerate() {
        Logger::info(tf!("check.circular.cycle_header", index + 1));

        if detail {
            print_detailed_cycle(cycle);
        } else {
            print_simple_cycle(cycle);
        }
        Logger::info("");
    }

    Logger::info(t!("check.circular.suggestion"));
    Ok(())
}

/// 打印详细循环路径
fn print_detailed_cycle(cycle: &[String]) {
    for (i, package) in cycle.iter().enumerate() {
        let next_package = &cycle[(i + 1) % cycle.len()];
        Logger::info(tf!("check.circular.cycle_detail", icons::ARROW, package, next_package));
    }
}

/// 打印简单循环路径
fn print_simple_cycle(cycle: &[String]) {
    let cycle_str = cycle.join(" → ");
    Logger::info(tf!("check.circular.cycle_simple", cycle_str));
}

/// 打印过期依赖表格
pub fn print_outdated_dependencies_table(
    outdated_deps: &[OutdatedDependency],
    detail: bool,
) -> Result<()> {
    Logger::info("");
    Logger::info(t!("check.outdated.details"));
    Logger::info("───────────────────────────────────────");

    if detail {
        print_detailed_outdated_deps(outdated_deps);
    } else {
        print_simple_outdated_deps(outdated_deps);
    }

    print_package_manager_suggestion();
    Ok(())
}

/// 打印详细的过期依赖信息
fn print_detailed_outdated_deps(outdated_deps: &[OutdatedDependency]) {
    let mut packages: BTreeMap<String, Vec<&OutdatedDependency>> = BTreeMap::new();
    for dep in outdated_deps {
        packages.entry(dep.package.clone()).or_default().push(dep);
    }

    for (package_name, deps) in packages {
        Logger::info(tf!("check.outdated.package_header", package_name));

        let mut unique_deps: BTreeMap<String, &OutdatedDependency> = BTreeMap::new();
        for dep in deps {
            unique_deps.insert(dep.name.clone(), dep);
        }

        for (_, dep) in unique_deps {
            Logger::info(tf!(
                "check.outdated.dep_detail_simple",
                Colors::info(&dep.name),
                dep.current,
                dep.latest,
                dep.dep_type
            ));
        }
        Logger::info("");
    }
}

/// 打印简单的过期依赖信息
fn print_simple_outdated_deps(outdated_deps: &[OutdatedDependency]) {
    let mut unique_deps: BTreeMap<String, (&OutdatedDependency, Vec<String>)> = BTreeMap::new();

    for dep in outdated_deps {
        unique_deps
            .entry(dep.name.clone())
            .and_modify(|(_, packages)| {
                if !packages.contains(&dep.package) {
                    packages.push(dep.package.clone());
                }
            })
            .or_insert((dep, vec![dep.package.clone()]));
    }

    for (_, (dep, packages)) in unique_deps {
        Logger::info(tf!(
            "check.outdated.dep_simple_single",
            Colors::info(&format!("[{}]", dep.name)),
            dep.current,
            dep.latest
        ));

        for package in &packages {
            Logger::info(format!("    {}", package));
        }
    }
    Logger::info("");
}

/// 打印包管理器建议
fn print_package_manager_suggestion() {
    let package_manager = Config::get_package_manager();
    let suggestion = match package_manager.as_str() {
        "pnpm" => t!("check.outdated.suggestion_pnpm"),
        "yarn" => t!("check.outdated.suggestion_yarn"),
        "npm" | _ => t!("check.outdated.suggestion_npm"),
    };
    Logger::info(suggestion);
}

/// 打印版本冲突表格
pub fn print_version_conflicts_table(conflicts: &[VersionConflict], detail: bool) -> Result<()> {
    Logger::info("");
    Logger::info(t!("check.versions.details"));
    Logger::info("───────────────────────────────────────");

    for (index, conflict) in conflicts.iter().enumerate() {
        Logger::info(tf!(
            "check.versions.conflict_header",
            index + 1,
            Colors::info(&conflict.name)
        ));

        if detail {
            print_detailed_conflict(conflict);
        } else {
            print_simple_conflict(conflict);
        }

        Logger::info(tf!(
            "check.versions.recommended",
            Colors::info(&conflict.recommended_version)
        ));
        Logger::info("");
    }

    Logger::info(t!("check.versions.suggestion"));
    Ok(())
}

/// 打印详细冲突信息
fn print_detailed_conflict(conflict: &VersionConflict) {
    for usage in &conflict.conflicts {
        Logger::info(tf!(
            "check.versions.usage_detail",
            usage.package,
            usage.version_spec,
            usage.resolved_version,
            usage.dep_type
        ));
    }
}

/// 打印简单冲突信息
fn print_simple_conflict(conflict: &VersionConflict) {
    let version_groups = group_by_version(&conflict.conflicts);

    for (version, usages) in version_groups {
        let packages: Vec<String> = usages.iter().map(|u| u.package.clone()).collect();
        Logger::info(tf!("check.versions.version_group", version, packages.join(", ")));
    }
}

/// 按版本分组
fn group_by_version(usages: &[ConflictUsage]) -> HashMap<String, Vec<&ConflictUsage>> {
    let mut unique_versions: HashMap<String, Vec<&ConflictUsage>> = HashMap::new();
    for usage in usages {
        unique_versions.entry(usage.resolved_version.clone()).or_default().push(usage);
    }
    unique_versions
}

/// 实时显示发现的过期包
pub fn print_outdated_package_realtime(dep_name: &str, current: &str, latest: &str, verbose: bool) {
    if verbose {
        Logger::warn(tf!("check.outdated.found_realtime", Colors::info(dep_name), current, latest));
    }
}
