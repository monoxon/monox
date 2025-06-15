// ============================================================================
// MonoX - CLI Analyze 命令
// ============================================================================
//
// 文件: src/cli/analyze.rs
// 职责: 依赖分析命令的 CLI 接口层
// 边界:
//   - ✅ 命令行参数定义和解析
//   - ✅ 调用核心分析器执行分析
//   - ✅ 结果格式化输出（表格/JSON）
//   - ✅ 用户交互和提示信息
//   - ❌ 不应包含依赖分析算法逻辑
//   - ❌ 不应包含配置文件加载逻辑
//   - ❌ 不应包含包扫描和解析逻辑
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

use anyhow::Result;
use clap::Args;
use serde_json;

use crate::core::DependencyAnalyzer;
use crate::models::config::Config;
use crate::utils::constants::icons;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 分析工作区依赖关系
#[derive(Debug, Args)]
pub struct AnalyzeArgs {
    /// 输出格式 (table, json)
    #[arg(short = 'f', long, default_value = "table")]
    pub format: String,

    /// 显示依赖详情
    #[arg(short = 'd', long)]
    pub detail: bool,

    /// 分析指定的单个包
    #[arg(short = 'p', long)]
    pub package: Option<String>,
}

pub fn handle_analyze(args: AnalyzeArgs) -> Result<()> {
    Logger::info(t!("cli.analyze.start"));

    // 获取工作区根目录（从全局配置中获取）
    let workspace_root = Config::get_workspace_root()?;

    // 获取verbose设置（从全局配置中获取）
    let verbose = Config::get_verbose()?;

    if !workspace_root.exists() {
        anyhow::bail!(tf!("error.workspace_not_exist", workspace_root.display()));
    }

    // 创建分析器并执行分析
    let mut analyzer = DependencyAnalyzer::new(workspace_root).with_verbose(verbose);

    let result = if let Some(package_name) = args.package {
        // 单包分析
        analyzer.analyze_single_package(&package_name)?
    } else {
        // 全工作区分析
        analyzer.analyze()?
    };

    // 输出结果
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&result)?;
            println!("{}", json_output);
        }
        "table" | _ => {
            print_table_format(&result, verbose, args.detail);
        }
    }

    Ok(())
}

fn print_table_format(
    result: &crate::models::DependencyAnalysisResult,
    verbose: bool,
    detail: bool,
) {
    // 使用像素图标而不是 emoji
    Logger::info(format!(
        "\n{} {}",
        icons::ANALYZE,
        t!("output.analysis_result")
    ));
    Logger::info("═══════════════════════════════════════");

    // 统计信息
    let stats = &result.statistics;
    Logger::info(format!(
        "{} {}",
        icons::PACKAGE,
        tf!("output.total_packages", stats.total_packages)
    ));
    Logger::info(format!(
        "{} {}",
        icons::STAGE,
        tf!("output.total_stages", stats.total_stages)
    ));
    Logger::info(format!(
        "{} {}",
        icons::DEPENDENCY,
        tf!(
            "output.packages_with_deps",
            stats.packages_with_workspace_deps
        )
    ));
    Logger::info(format!(
        "{} {}",
        icons::TIME,
        tf!("output.analysis_duration", stats.analysis_duration_ms)
    ));

    // 循环依赖检查
    if !result.circular_dependencies.is_empty() {
        Logger::info(format!(
            "\n{} {}",
            icons::ERROR,
            t!("output.circular_dependencies")
        ));
        Logger::info("───────────────────────────────────────");
        for (i, cycle) in result.circular_dependencies.iter().enumerate() {
            Logger::info(format!("{}. {}", i + 1, cycle.join(" → ")));
        }
    } else {
        Logger::info(format!(
            "\n{} {}",
            icons::SUCCESS,
            t!("output.no_circular_dependencies")
        ));
    }

    // 构建阶段
    if !result.stages.is_empty() {
        Logger::info(format!("\n{} {}", icons::STAGE, t!("output.build_stages")));
        Logger::info("───────────────────────────────────────");
        for (stage_idx, stage) in result.stages.iter().enumerate() {
            Logger::info(tf!("output.stage_info", stage_idx + 1, stage.len()));
            for package in stage {
                if detail {
                    // 详细模式：显示依赖信息
                    if package.workspace_dependencies.is_empty() {
                        Logger::info(format!(
                            "  {} {} ({})",
                            icons::PACKAGE,
                            package.name,
                            t!("output.no_workspace_deps")
                        ));
                    } else {
                        // 使用列表显示依赖项
                        Logger::info(format!(
                            "  {} {} ({})",
                            icons::PACKAGE,
                            package.name,
                            tf!(
                                "output.depends_on_count",
                                package.workspace_dependencies.len()
                            )
                        ));
                        for dep in &package.workspace_dependencies {
                            Logger::info(format!("    {} {}", icons::DEPENDENCY, dep));
                        }
                    }
                } else {
                    // 简洁模式：只显示包名
                    Logger::info(format!("  {} {}", icons::PACKAGE, package.name));
                }

                if verbose {
                    Logger::info(format!(
                        "    {}",
                        tf!("output.path", package.folder.display())
                    ));
                    Logger::info(format!("    {}", tf!("output.version", package.version)));
                    if !package.scripts.is_empty() {
                        let scripts: Vec<String> = package.scripts.keys().cloned().collect();
                        Logger::info(format!("    {}", tf!("output.scripts", scripts.join(", "))));
                    }
                }
            }
            Logger::info("");
        }
    }

    // 包详情（仅在详细模式下）
    if verbose {
        Logger::info(format!(
            "{} {}",
            icons::PACKAGE,
            t!("output.package_details")
        ));
        Logger::info("───────────────────────────────────────");
        for package in &result.packages {
            Logger::info(format!(
                "{} {} v{}",
                icons::PACKAGE,
                package.name,
                package.version
            ));
            Logger::info(format!(
                "  {}",
                tf!("output.path", package.folder.display())
            ));

            if !package.dependencies.is_empty() {
                Logger::info(format!(
                    "  {}",
                    tf!("output.all_dependencies", package.dependencies.len())
                ));
                for (dep_name, version) in &package.dependencies {
                    let marker = if package.workspace_dependencies.contains(dep_name) {
                        icons::DEPENDENCY
                    } else {
                        icons::PACKAGE
                    };
                    Logger::info(format!("    {} {} {}", marker, dep_name, version));
                }
            }

            if !package.scripts.is_empty() {
                Logger::info(format!("  {}", t!("output.scripts_detail")));
                for (script_name, script_command) in &package.scripts {
                    Logger::info(format!("    {} = {}", script_name, script_command));
                }
            }
            Logger::info("");
        }
    }

    Logger::info(format!("{} {}", icons::INFO, t!("output.usage_tip")));
}
