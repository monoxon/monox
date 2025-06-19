// ============================================================================
// MonoX - CLI Fix 命令
// ============================================================================
//
// 文件: src/cli/fix.rs
// 职责: 版本冲突自动修复命令的 CLI 接口层
// 边界:
//   - ✅ 命令行参数定义和解析
//   - ✅ 调用版本冲突修复器执行修复
//   - ✅ 修复结果格式化输出
//   - ✅ 用户交互和确认提示
//   - ❌ 不应包含具体修复逻辑
//   - ❌ 不应包含文件读写逻辑
//   - ❌ 不应包含版本计算逻辑
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::checker::{HealthChecker, VersionConflict};
use crate::models::config::Config;
use crate::utils::colors::Colors;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 修复结果信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    /// 修复的包名
    pub package: String,
    /// 修复的依赖名
    pub dependency: String,
    /// 原版本
    pub old_version: String,
    /// 新版本
    pub new_version: String,
    /// 依赖类型
    pub dep_type: String,
}

/// 自动修复版本冲突
#[derive(Debug, Args)]
pub struct FixArgs {
    /// 只检查，不实际修复（预演模式）
    #[arg(long)]
    pub dry_run: bool,

    /// 自动确认所有修复操作
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// 输出格式 (table, json)
    #[arg(short = 'f', long, default_value = "table")]
    pub format: String,

    /// 显示详细信息
    #[arg(short = 'd', long)]
    pub detail: bool,
}

pub fn handle_fix(args: FixArgs) -> Result<()> {
    Logger::info(t!("cli.fix.start"));

    // 获取工作区根目录
    let workspace_root = Config::get_workspace_root();
    let verbose = Config::get_verbose();

    if !workspace_root.exists() {
        anyhow::bail!(tf!("error.workspace_not_exist", workspace_root.display()));
    }

    // 创建健康检查器
    let checker = HealthChecker::new(workspace_root.clone()).with_verbose(verbose);

    // 收集所有未被忽略的 package.json 文件
    let package_files = collect_package_files(&workspace_root, verbose)?;

    if package_files.is_empty() {
        Logger::info(t!("fix.no_packages_found"));
        return Ok(());
    }

    // 收集版本冲突
    let version_conflicts = checker.check_version_conflicts()?;

    if version_conflicts.is_empty() {
        Logger::success(t!("fix.no_conflicts_found"));
        return Ok(());
    }

    Logger::info(tf!("fix.conflicts_found", version_conflicts.len()));

    // 计算修复方案
    let fix_plan = calculate_fix_plan(&version_conflicts, &package_files)?;

    if fix_plan.is_empty() {
        Logger::info(t!("fix.no_fixes_needed"));
        return Ok(());
    }

    // 显示修复方案
    display_fix_plan(&fix_plan, &args)?;

    if args.dry_run {
        Logger::info(t!("fix.dry_run_complete"));
        return Ok(());
    }

    // 确认修复操作
    if !args.yes && !confirm_fix()? {
        Logger::info(t!("fix.cancelled"));
        return Ok(());
    }

    // 执行修复
    let results = execute_fixes(&fix_plan, &package_files, verbose)?;

    // 显示修复结果
    display_fix_results(&results, &args)?;

    Logger::success(tf!("fix.completed", results.len()));

    Ok(())
}

/// 收集 package.json 文件
fn collect_package_files(workspace_root: &Path, verbose: bool) -> Result<Vec<std::path::PathBuf>> {
    let mut package_files = Vec::new();

    fn scan_directory(
        dir: &Path,
        package_files: &mut Vec<std::path::PathBuf>,
        verbose: bool,
    ) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let relative_path = path
                    .strip_prefix(dir.parent().unwrap_or(dir))
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                // 检查是否应该忽略此路径
                if Config::should_ignore_path(&relative_path).unwrap_or(false) {
                    if verbose {
                        Logger::info(tf!("fix.skipping_path", &relative_path));
                    }
                    continue;
                }

                if path.is_dir() {
                    scan_directory(&path, package_files, verbose)?;
                } else if path.file_name() == Some(std::ffi::OsStr::new("package.json")) {
                    package_files.push(path);
                }
            }
        }
        Ok(())
    }

    scan_directory(workspace_root, &mut package_files, verbose)?;

    if verbose {
        Logger::info(tf!("fix.found_package_files", package_files.len()));
    }

    Ok(package_files)
}

/// 计算修复方案
fn calculate_fix_plan(
    conflicts: &[VersionConflict],
    package_files: &[std::path::PathBuf],
) -> Result<Vec<FixResult>> {
    let mut fixes = Vec::new();

    // 为每个包文件建立路径映射
    let mut package_path_map: HashMap<String, &std::path::PathBuf> = HashMap::new();

    for package_file in package_files {
        let content = fs::read_to_string(package_file)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(name) = package_json["name"].as_str() {
            package_path_map.insert(name.to_string(), package_file);
        }
    }

    for conflict in conflicts {
        let recommended_version = &conflict.recommended_version;

        for usage in &conflict.conflicts {
            // 如果当前版本不等于推荐版本，需要修复
            if usage.resolved_version != *recommended_version {
                // 保持原有的版本前缀格式
                let new_version = preserve_version_format(&usage.version_spec, recommended_version);

                let fix = FixResult {
                    package: usage.package.clone(),
                    dependency: conflict.name.clone(),
                    old_version: usage.version_spec.clone(),
                    new_version,
                    dep_type: usage.dep_type.clone(),
                };

                fixes.push(fix);
            }
        }
    }

    Ok(fixes)
}

/// 保持原有版本格式，只替换版本号
fn preserve_version_format(original_spec: &str, new_version: &str) -> String {
    // 检测原有版本的前缀
    if original_spec.starts_with("^") {
        format!("^{}", new_version)
    } else if original_spec.starts_with("~") {
        format!("~{}", new_version)
    } else if original_spec.starts_with(">=") {
        format!(">={}", new_version)
    } else if original_spec.starts_with("<=") {
        format!("<={}", new_version)
    } else if original_spec.starts_with(">") {
        format!(">{}", new_version)
    } else if original_spec.starts_with("<") {
        format!("<{}", new_version)
    } else if original_spec.starts_with("=") {
        format!("={}", new_version)
    } else {
        // 没有前缀，直接使用版本号
        new_version.to_string()
    }
}

/// 显示修复方案
fn display_fix_plan(fixes: &[FixResult], args: &FixArgs) -> Result<()> {
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::json!({
                "fix_plan": fixes,
                "count": fixes.len()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "table" | _ => {
            print_fix_plan_table(fixes, args.detail)?;
        }
    }
    Ok(())
}

/// 打印修复方案表格
fn print_fix_plan_table(fixes: &[FixResult], detail: bool) -> Result<()> {
    Logger::info("");
    Logger::info(t!("fix.plan_details"));
    Logger::info("───────────────────────────────────────");

    if detail {
        // 详细模式：显示每个修复操作
        for (index, fix) in fixes.iter().enumerate() {
            Logger::info(tf!(
                "fix.fix_detail",
                index + 1,
                Colors::info(&fix.package),
                fix.dependency,
                fix.old_version,
                fix.new_version,
                fix.dep_type
            ));
        }
    } else {
        // 简单模式：按包分组显示
        let mut packages: HashMap<String, Vec<&FixResult>> = HashMap::new();
        for fix in fixes {
            packages.entry(fix.package.clone()).or_default().push(fix);
        }

        for (package_name, package_fixes) in packages {
            Logger::info(tf!("fix.package_header", package_name));
            for fix in package_fixes {
                Logger::info(tf!(
                    "fix.fix_simple",
                    fix.dependency,
                    fix.old_version,
                    fix.new_version
                ));
            }
            Logger::info("");
        }
    }

    Logger::info(tf!("fix.total_fixes", fixes.len()));
    Ok(())
}

/// 确认修复操作
fn confirm_fix() -> Result<bool> {
    use std::io::{self, Write};

    print!("{} ", t!("fix.confirm_prompt"));
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes" || input == "是" || input == "确认")
}

/// 执行修复操作
fn execute_fixes(
    fixes: &[FixResult],
    package_files: &[std::path::PathBuf],
    verbose: bool,
) -> Result<Vec<FixResult>> {
    let mut executed_fixes = Vec::new();
    let mut package_path_map: HashMap<String, &std::path::PathBuf> = HashMap::new();

    // 建立包名到文件路径的映射
    for package_file in package_files {
        let content = fs::read_to_string(package_file)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(name) = package_json["name"].as_str() {
            package_path_map.insert(name.to_string(), package_file);
        }
    }

    // 按包分组执行修复
    let mut packages: HashMap<String, Vec<&FixResult>> = HashMap::new();
    for fix in fixes {
        packages.entry(fix.package.clone()).or_default().push(fix);
    }

    for (package_name, package_fixes) in packages {
        if let Some(&package_file) = package_path_map.get(&package_name) {
            if verbose {
                Logger::info(tf!("fix.processing_package", package_name));
            }

            // 读取原始文件内容
            let mut content = fs::read_to_string(package_file)?;

            // 验证文件格式
            let package_json: serde_json::Value = serde_json::from_str(&content)?;

            // 逐个应用修复
            for fix in &package_fixes {
                // 检查依赖是否存在
                if let Some(deps) = package_json[&fix.dep_type].as_object() {
                    if deps.contains_key(&fix.dependency) {
                        // 执行精确的文本替换
                        if replace_dependency_version(
                            &mut content,
                            &fix.dep_type,
                            &fix.dependency,
                            &fix.old_version,
                            &fix.new_version,
                        )? {
                            executed_fixes.push((*fix).clone());

                            if verbose {
                                Logger::info(tf!(
                                    "fix.updated_dependency",
                                    fix.dependency,
                                    fix.old_version,
                                    fix.new_version
                                ));
                            }
                        }
                    }
                }
            }

            // 写回文件
            fs::write(package_file, content)?;

            if verbose {
                Logger::info(tf!("fix.updated_package", package_name));
            }
        }
    }

    Ok(executed_fixes)
}

/// 精确替换依赖版本，保持文件格式
fn replace_dependency_version(
    content: &mut String,
    _dep_type: &str,
    dependency: &str,
    old_version: &str,
    new_version: &str,
) -> Result<bool> {
    // 构建精确的搜索模式
    // 匹配：  "dependency": "old_version"
    let search_pattern = format!(r#""{}":\s*"{}""#, dependency, regex::escape(old_version));
    let replacement = format!(r#""{}": "{}""#, dependency, new_version);

    // 使用正则表达式进行替换
    let regex = regex::Regex::new(&search_pattern)?;

    if regex.is_match(content) {
        let new_content = regex.replace(content, replacement.as_str());
        *content = new_content.to_string();
        return Ok(true);
    }

    Ok(false)
}

/// 显示修复结果
fn display_fix_results(results: &[FixResult], args: &FixArgs) -> Result<()> {
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::json!({
                "fix_results": results,
                "count": results.len()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "table" | _ => {
            print_fix_results_table(results)?;
        }
    }
    Ok(())
}

/// 打印修复结果表格
fn print_fix_results_table(results: &[FixResult]) -> Result<()> {
    Logger::info("");
    Logger::info(t!("fix.results_details"));
    Logger::info("───────────────────────────────────────");

    let mut packages: HashMap<String, Vec<&FixResult>> = HashMap::new();
    for result in results {
        packages
            .entry(result.package.clone())
            .or_default()
            .push(result);
    }

    for (package_name, package_results) in packages {
        Logger::info(tf!("fix.package_header", Colors::info(&package_name)));
        for result in package_results {
            Logger::info(tf!(
                "fix.result_detail",
                result.dependency,
                result.old_version,
                result.new_version
            ));
        }
        Logger::info("");
    }

    Ok(())
}
