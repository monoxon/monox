// ============================================================================
// MonoX - CLI Update 命令
// ============================================================================
//
// 文件: src/cli/update.rs
// 职责: 依赖更新命令的 CLI 接口层
// 边界:
//   - ✅ 命令行参数定义和解析
//   - ✅ 调用核心更新器执行更新
//   - ✅ 用户交互和确认提示
//   - ❌ 不应包含具体依赖更新逻辑
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

use anyhow::Result;
use clap::Args;
use regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::checker::HealthChecker;
use crate::models::config::Config;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 更新结果信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    /// 更新的依赖名
    pub dependency: String,
    /// 原版本
    pub old_version: String,
    /// 新版本
    pub new_version: String,
    /// 所在的包名
    pub package: String,
    /// 依赖类型
    pub dep_type: String,
}

/// 更新依赖命令
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// 要更新的依赖名称
    #[arg(short = 'p', long)]
    pub package: Option<String>,

    /// 更新所有过期依赖
    #[arg(short = 'a', long)]
    pub all: bool,

    /// 指定要更新到的版本 (不指定则使用最新版本)
    #[arg(long)]
    pub version: Option<String>,

    /// 只检查，不实际更新（预演模式）
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn handle_update(args: UpdateArgs) -> Result<()> {
    Logger::info(t!("cli.update.start"));

    // 验证参数
    if !args.all && args.package.is_none() {
        anyhow::bail!(t!("update.missing_package_or_all"));
    }

    // 获取工作区根目录
    let workspace_root = Config::get_workspace_root();
    let verbose = Config::get_verbose();

    if !workspace_root.exists() {
        anyhow::bail!(tf!("error.workspace_not_exist", workspace_root.display()));
    }

    // 收集所有未被忽略的 package.json 文件
    let package_files = collect_package_files(&workspace_root, verbose)?;

    if package_files.is_empty() {
        Logger::info(t!("update.no_packages_found"));
        return Ok(());
    }

    let update_plan = if args.all {
        // 更新所有过期依赖
        create_update_plan_for_all(&package_files, verbose).await?
    } else {
        // 更新指定依赖
        let dependency_name = args.package.unwrap();
        create_update_plan_for_dependency(&package_files, &dependency_name, args.version.as_deref())
            .await?
    };

    if update_plan.is_empty() {
        if args.all {
            Logger::success(t!("update.no_outdated_found"));
        } else {
            Logger::info(t!("update.dependency_not_found"));
        }
        return Ok(());
    }

    if args.dry_run {
        // 预演模式：显示更新方案
        display_update_plan(&update_plan)?;
        Logger::info(t!("update.dry_run_complete"));
        return Ok(());
    }

    // 执行更新
    let results = execute_updates(&update_plan, &package_files, verbose)?;

    // 显示更新结果
    display_update_results(&results)?;

    Logger::success(tf!("update.completed", results.len()));

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
                        Logger::info(tf!("update.skipping_path", &relative_path));
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
        Logger::info(tf!("update.found_package_files", package_files.len()));
    }

    Ok(package_files)
}

/// 为所有过期依赖创建更新方案
async fn create_update_plan_for_all(
    _package_files: &[std::path::PathBuf],
    verbose: bool,
) -> Result<Vec<UpdateResult>> {
    Logger::info(t!("update.checking_outdated"));

    // 使用 HealthChecker 检查过期依赖
    let workspace_root = Config::get_workspace_root();
    let checker = HealthChecker::new(workspace_root).with_verbose(verbose);

    let (outdated_deps, _) = checker
        .check_outdated_dependencies_with_progress(None)
        .await?;

    let mut updates = Vec::new();
    for outdated_dep in outdated_deps {
        updates.push(UpdateResult {
            dependency: outdated_dep.name,
            old_version: outdated_dep.current,
            new_version: outdated_dep.latest,
            package: outdated_dep.package,
            dep_type: outdated_dep.dep_type,
        });
    }

    Ok(updates)
}

/// 为指定依赖创建更新方案
async fn create_update_plan_for_dependency(
    package_files: &[std::path::PathBuf],
    dependency_name: &str,
    target_version: Option<&str>,
) -> Result<Vec<UpdateResult>> {
    Logger::info(tf!("update.checking_dependency", dependency_name));

    let mut updates = Vec::new();

    // 确定目标版本
    let new_version = if let Some(version) = target_version {
        version.to_string()
    } else {
        // 获取最新版本
        match get_latest_version_async(dependency_name).await? {
            Some(version) => version,
            None => {
                Logger::warn(tf!("update.version_fetch_failed", dependency_name));
                return Ok(updates);
            }
        }
    };

    // 在所有 package.json 中查找该依赖
    for package_file in package_files {
        let content = fs::read_to_string(package_file)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        let package_name = package_json["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // 检查所有依赖类型
        let dep_types = ["dependencies", "devDependencies", "peerDependencies"];

        for dep_type in &dep_types {
            if let Some(deps) = package_json[dep_type].as_object() {
                if let Some(current_version_value) = deps.get(dependency_name) {
                    let current_version = current_version_value.as_str().unwrap_or("").to_string();

                    // 跳过工作区依赖、文件依赖等
                    if should_skip_dependency(&current_version) {
                        continue;
                    }

                    // 提取当前版本号
                    let clean_current = extract_version_from_spec(&current_version);

                    // 如果版本不同，添加到更新列表
                    if clean_current != new_version {
                        updates.push(UpdateResult {
                            dependency: dependency_name.to_string(),
                            old_version: current_version,
                            new_version: new_version.clone(),
                            package: package_name.clone(),
                            dep_type: dep_type.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(updates)
}

/// 异步获取最新版本
async fn get_latest_version_async(package_name: &str) -> Result<Option<String>> {
    use tokio::process::Command;

    let output = Command::new("npm")
        .args(&["view", package_name, "version", "--json"])
        .output()
        .await?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    if trimmed.is_empty() {
        return Ok(None);
    }

    // 解析响应
    if let Ok(response) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(version) = response.as_str() {
            return Ok(Some(version.to_string()));
        }
    }

    // 后备处理：直接使用去引号的字符串
    let version = trimmed.trim_matches('"');
    Ok(Some(version.to_string()))
}

/// 检查是否应该跳过依赖检查
fn should_skip_dependency(version_spec: &str) -> bool {
    version_spec.starts_with("workspace:")
        || version_spec.starts_with("file:")
        || version_spec.starts_with("link:")
        || version_spec.contains("git+")
        || version_spec.contains("github:")
}

/// 从版本规范中提取版本号
fn extract_version_from_spec(version_spec: &str) -> String {
    version_spec
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches("<=")
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .to_string()
}

/// 显示更新方案
fn display_update_plan(updates: &[UpdateResult]) -> Result<()> {
    use crate::utils::colors::Colors;

    Logger::info(t!("update.plan_details"));
    Logger::info("═══════════════════════════════════════");

    // 按包分组显示
    let mut packages: HashMap<String, Vec<&UpdateResult>> = HashMap::new();
    for update in updates {
        packages
            .entry(update.package.clone())
            .or_default()
            .push(update);
    }

    for (package_name, package_updates) in packages {
        Logger::info(tf!("update.package_header", package_name));
        for update in package_updates {
            let old_version = Colors::red(&update.old_version);
            let new_version = Colors::green(&update.new_version);
            Logger::info(tf!(
                "update.update_simple",
                update.dependency,
                old_version,
                new_version
            ));
        }
        Logger::info("");
    }

    Logger::info(tf!("update.total_updates", updates.len()));
    Ok(())
}

/// 执行更新
fn execute_updates(
    updates: &[UpdateResult],
    package_files: &[std::path::PathBuf],
    verbose: bool,
) -> Result<Vec<UpdateResult>> {
    let mut executed_updates = Vec::new();
    let mut package_path_map: HashMap<String, &std::path::PathBuf> = HashMap::new();

    // 建立包名到文件路径的映射
    for package_file in package_files {
        let content = fs::read_to_string(package_file)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(name) = package_json["name"].as_str() {
            package_path_map.insert(name.to_string(), package_file);
        }
    }

    // 按包分组执行更新
    let mut packages: HashMap<String, Vec<&UpdateResult>> = HashMap::new();
    for update in updates {
        packages
            .entry(update.package.clone())
            .or_default()
            .push(update);
    }

    for (package_name, package_updates) in packages {
        if let Some(&package_file) = package_path_map.get(&package_name) {
            if verbose {
                Logger::info(tf!("update.processing_package", package_name));
            }

            let mut content = fs::read_to_string(package_file)?;
            let mut updated_count = 0;

            for update in &package_updates {
                if replace_dependency_version(
                    &mut content,
                    &update.dependency,
                    &update.old_version,
                    &update.new_version,
                )? {
                    updated_count += 1;
                    executed_updates.push((*update).clone());

                    if verbose {
                        Logger::info(tf!(
                            "update.updated_dependency",
                            update.dependency,
                            update.old_version,
                            update.new_version
                        ));
                    }
                }
            }

            if updated_count > 0 {
                fs::write(package_file, content)?;
                if verbose {
                    Logger::info(tf!("update.updated_package", package_name));
                }
            }
        }
    }

    Ok(executed_updates)
}

/// 替换依赖版本
fn replace_dependency_version(
    content: &mut String,
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

/// 显示更新结果
fn display_update_results(results: &[UpdateResult]) -> Result<()> {
    use crate::utils::colors::Colors;

    Logger::info(t!("update.results_details"));
    Logger::info("═══════════════════════════════════════");

    // 按包分组显示结果
    let mut packages: HashMap<String, Vec<&UpdateResult>> = HashMap::new();
    for result in results {
        packages
            .entry(result.package.clone())
            .or_default()
            .push(result);
    }

    for (package_name, package_results) in packages {
        Logger::info(tf!("update.package_header", package_name));
        for result in package_results {
            let old_version = Colors::red(&result.old_version);
            let new_version = Colors::green(&result.new_version);
            Logger::info(tf!(
                "update.result_detail",
                result.dependency,
                old_version,
                new_version
            ));
        }
        Logger::info("");
    }

    Ok(())
}
