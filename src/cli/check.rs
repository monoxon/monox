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
//   - ❌ 不应包含具体检查逻辑
//   - ❌ 不应包含文件扫描逻辑
//   - ❌ 不应包含规则定义
//   - ❌ 不应包含数据模型定义
//
// ============================================================================

use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::core::analyzer::DependencyAnalyzer;
use crate::models::config::Config;
use crate::utils::colors::Colors;
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 过期依赖信息
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    /// 依赖包名
    pub name: String,
    /// 冲突的版本使用情况
    pub conflicts: Vec<ConflictUsage>,
    /// 推荐的统一版本
    pub recommended_version: String,
}

/// 版本冲突使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 依赖信息
#[derive(Debug, Clone)]
struct DependencyInfo {
    /// 包名
    name: String,
    /// 版本规范
    version_spec: String,
    /// 使用该依赖的包列表
    used_by: Vec<(String, String)>, // (package_name, dep_type)
}

/// npm view 命令的响应结构
#[derive(Debug, Deserialize)]
struct NpmViewResponse {
    version: String,
}

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

pub fn handle_check(args: CheckArgs) -> Result<()> {
    Logger::info(t!("cli.check.start"));

    // 获取工作区根目录（从全局配置中获取）
    let workspace_root = Config::get_workspace_root();
    let verbose = Config::get_verbose();

    if !workspace_root.exists() {
        anyhow::bail!(tf!("error.workspace_not_exist", workspace_root.display()));
    }

    // 如果没有指定任何检查选项，默认检查循环依赖
    let check_circular = args.circular || (!args.versions && !args.outdated);
    let check_versions = args.versions;
    let check_outdated = args.outdated;

    let mut has_issues = false;

    // 检查循环依赖
    if check_circular {
        has_issues |= check_circular_dependencies(&workspace_root, verbose, &args)?;
    }

    // 检查版本冲突
    if check_versions {
        has_issues |= check_version_conflicts(&workspace_root, verbose, &args)?;
    }

    // 检查过期依赖
    if check_outdated {
        has_issues |= check_outdated_dependencies(&workspace_root, verbose, &args)?;
    }

    // 输出总结
    if has_issues {
        Logger::error(t!("check.issues_found"));
        std::process::exit(1);
    } else {
        Logger::success(t!("check.all_good"));
    }

    Ok(())
}

/// 检查循环依赖
fn check_circular_dependencies(
    workspace_root: &std::path::Path,
    verbose: bool,
    args: &CheckArgs,
) -> Result<bool> {
    if verbose {
        Logger::info(t!("check.circular.start"));
    }

    // 创建分析器并执行分析
    let mut analyzer = DependencyAnalyzer::new(workspace_root.to_path_buf()).with_verbose(verbose);
    let result = analyzer.analyze_workspace()?;

    if result.circular_dependencies.is_empty() {
        Logger::success(t!("check.circular.none_found"));
        return Ok(false);
    }

    // 发现循环依赖
    Logger::error(tf!(
        "check.circular.found",
        result.circular_dependencies.len()
    ));

    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::json!({
                "circular_dependencies": result.circular_dependencies,
                "count": result.circular_dependencies.len()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "table" | _ => {
            print_circular_dependencies_table(&result.circular_dependencies, args.detail);
        }
    }

    Ok(true)
}

/// 检查版本冲突
fn check_version_conflicts(
    workspace_root: &std::path::Path,
    verbose: bool,
    args: &CheckArgs,
) -> Result<bool> {
    if verbose {
        Logger::info(t!("check.versions.start"));
    }

    // 收集所有未被忽略的 package.json 文件
    let package_files = collect_package_files(workspace_root, verbose)?;

    if package_files.is_empty() {
        Logger::success(t!("check.versions.none_found"));
        return Ok(false);
    }

    // 收集所有依赖并按包名分组
    let version_conflicts = collect_version_conflicts(&package_files, verbose)?;

    if version_conflicts.is_empty() {
        Logger::success(t!("check.versions.none_found"));
        return Ok(false);
    }

    // 发现版本冲突
    Logger::error(tf!("check.versions.found", version_conflicts.len()));

    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::json!({
                "version_conflicts": version_conflicts,
                "count": version_conflicts.len()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "table" | _ => {
            print_version_conflicts_table(&version_conflicts, args.detail)?;
        }
    }

    Ok(true)
}

/// 检查过期依赖
fn check_outdated_dependencies(
    workspace_root: &std::path::Path,
    verbose: bool,
    args: &CheckArgs,
) -> Result<bool> {
    use crate::utils::spinner::Spinner;

    if verbose {
        Logger::info(t!("check.outdated.start"));
    }

    // 收集所有未被忽略的 package.json 文件
    let package_files = collect_package_files(workspace_root, verbose)?;

    if package_files.is_empty() {
        Logger::success(t!("check.outdated.none_found"));
        return Ok(false);
    }

    // 收集并去重所有依赖
    let unique_dependencies = collect_unique_dependencies(&package_files, verbose)?;

    if unique_dependencies.is_empty() {
        Logger::success(t!("check.outdated.none_found"));
        return Ok(false);
    }

    if verbose {
        Logger::info(tf!(
            "check.outdated.collected_dependencies",
            unique_dependencies.len()
        ));
    }

    // 启动 spinner
    let mut spinner = Spinner::new(tf!(
        "check.outdated.checking_dependencies",
        unique_dependencies.len()
    ));
    if !verbose {
        spinner.start();
    }

    // 实时显示过期依赖的共享状态
    let outdated_deps = Arc::new(Mutex::new(Vec::new()));
    let checked_count = Arc::new(Mutex::new(0));
    let total_count = unique_dependencies.len();
    let found_packages = Arc::new(Mutex::new(std::collections::HashSet::<String>::new()));

    // 创建一个用于实时更新显示的线程
    let display_outdated = Arc::clone(&outdated_deps);
    let display_checked = Arc::clone(&checked_count);
    let display_found = Arc::clone(&found_packages);
    let display_spinner = if !verbose {
        let spinner_clone = Arc::new(Mutex::new(spinner));
        let spinner_ref = Arc::clone(&spinner_clone);

        let handle = thread::spawn(move || loop {
            let checked = *display_checked.lock().unwrap();
            let outdated_count = display_found.lock().unwrap().len();

            if checked >= total_count {
                break;
            }

            let message = if outdated_count > 0 {
                tf!(
                    "check.outdated.progress_with_found",
                    checked,
                    total_count,
                    outdated_count
                )
            } else {
                tf!("check.outdated.progress", checked, total_count)
            };

            if let Ok(spinner) = spinner_ref.lock() {
                spinner.update_message(message);
            }

            thread::sleep(std::time::Duration::from_millis(50));
        });
        Some((spinner_clone, handle))
    } else {
        None
    };

    // 使用多线程并发检查依赖
    let mut handles = Vec::new();
    let thread_count = calculate_optimal_thread_count(unique_dependencies.len());
    let chunk_size = std::cmp::max(1, unique_dependencies.len() / thread_count);
    let dependencies_vec: Vec<_> = unique_dependencies.into_iter().collect();

    if verbose {
        Logger::info(tf!("check.outdated.using_threads", thread_count));
    }

    for chunk in dependencies_vec.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let outdated_deps = Arc::clone(&outdated_deps);
        let checked_count = Arc::clone(&checked_count);
        let found_packages = Arc::clone(&found_packages);

        let handle = thread::spawn(move || {
            for (dep_name, dep_info) in chunk {
                if let Ok(Some(latest_version)) = get_latest_version(&dep_name, verbose) {
                    let current_version = extract_version_from_spec(&dep_info.version_spec);

                    if current_version != latest_version
                        && !is_version_satisfied(&current_version, &latest_version)
                    {
                        // 记录发现的过期包（用于去重显示）
                        let mut found_set = found_packages.lock().unwrap();
                        let is_new_package = found_set.insert(dep_name.clone());
                        drop(found_set);

                        // 实时显示新发现的过期依赖
                        if is_new_package {
                            print_outdated_package_realtime(
                                &dep_name,
                                &current_version,
                                &latest_version,
                                verbose,
                            );
                        }

                        // 为每个使用该依赖的包创建过期依赖记录
                        for (package_name, dep_type) in &dep_info.used_by {
                            let outdated = OutdatedDependency {
                                name: dep_name.clone(),
                                current: current_version.clone(),
                                latest: latest_version.clone(),
                                package: package_name.clone(),
                                dep_type: dep_type.clone(),
                            };

                            outdated_deps.lock().unwrap().push(outdated);
                        }
                    }
                }

                // 更新已检查计数
                let mut count = checked_count.lock().unwrap();
                *count += 1;
            }
        });

        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        let _ = handle.join();
    }

    // 停止显示线程和 spinner
    if let Some((spinner_arc, display_handle)) = display_spinner {
        let _ = display_handle.join();
        if let Ok(mut spinner) = spinner_arc.lock() {
            spinner.stop();
        }
    }

    let outdated_deps = outdated_deps.lock().unwrap().clone();
    let unique_outdated_count = found_packages.lock().unwrap().len();

    if outdated_deps.is_empty() {
        Logger::success(t!("check.outdated.none_found"));
        return Ok(false);
    }

    // 发现过期依赖 - 显示去重后的包数量和总依赖实例数量
    if unique_outdated_count == outdated_deps.len() {
        // 如果去重数量等于总数量，只显示一个数字
        Logger::error(tf!("check.outdated.found", unique_outdated_count));
    } else {
        // 如果不同，显示两个数字：去重包数量和总依赖实例数量
        Logger::error(tf!(
            "check.outdated.found_with_instances",
            unique_outdated_count,
            outdated_deps.len()
        ));
    }

    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::json!({
                "outdated_dependencies": outdated_deps,
                "count": outdated_deps.len(),
                "unique_count": unique_outdated_count
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
        "table" | _ => {
            print_outdated_dependencies_table(&outdated_deps, args.detail)?;
        }
    }

    Ok(true)
}

/// 收集所有未被忽略的 package.json 文件
fn collect_package_files(
    workspace_root: &std::path::Path,
    verbose: bool,
) -> Result<Vec<std::path::PathBuf>> {
    use crate::models::config::Config;
    use std::fs;

    let mut package_files = Vec::new();

    fn scan_directory(
        dir: &std::path::Path,
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
                        Logger::info(tf!("check.outdated.skipping_path", &relative_path));
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
        Logger::info(tf!(
            "check.outdated.found_package_files",
            package_files.len()
        ));
    }

    Ok(package_files)
}

/// 收集并去重所有依赖
fn collect_unique_dependencies(
    package_files: &[std::path::PathBuf],
    verbose: bool,
) -> Result<std::collections::BTreeMap<String, DependencyInfo>> {
    use std::collections::BTreeMap;
    use std::fs;

    let mut unique_dependencies: BTreeMap<String, DependencyInfo> = BTreeMap::new();

    for package_file in package_files {
        if verbose {
            Logger::info(tf!(
                "check.outdated.processing_package",
                package_file.display()
            ));
        }

        let content = fs::read_to_string(package_file)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        let package_name = package_json["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // 处理不同类型的依赖
        let dep_types = ["dependencies", "devDependencies", "peerDependencies"];

        for dep_type in &dep_types {
            if let Some(deps) = package_json[dep_type].as_object() {
                for (dep_name, version_value) in deps {
                    let version_spec = version_value.as_str().unwrap_or("").to_string();

                    // 跳过特殊依赖
                    if should_skip_dependency(&version_spec) {
                        continue;
                    }

                    // 添加或更新依赖信息
                    unique_dependencies
                        .entry(dep_name.clone())
                        .and_modify(|dep_info| {
                            dep_info
                                .used_by
                                .push((package_name.clone(), dep_type.to_string()));
                        })
                        .or_insert_with(|| DependencyInfo {
                            name: dep_name.clone(),
                            version_spec: version_spec.clone(),
                            used_by: vec![(package_name.clone(), dep_type.to_string())],
                        });
                }
            }
        }
    }

    if verbose {
        Logger::info(tf!(
            "check.outdated.unique_dependencies_count",
            unique_dependencies.len()
        ));
    }

    Ok(unique_dependencies)
}

/// 检查单个依赖的版本
fn check_dependency_version(
    dep_name: &str,
    version_spec: &str,
    package_name: &str,
    dep_type: &str,
    checked_packages: &mut HashMap<String, String>,
    verbose: bool,
) -> Result<Option<OutdatedDependency>> {
    // 跳过工作区依赖
    if version_spec.starts_with("workspace:") {
        return Ok(None);
    }

    // 跳过文件路径依赖
    if version_spec.starts_with("file:") || version_spec.starts_with("link:") {
        return Ok(None);
    }

    // 跳过 git 依赖
    if version_spec.contains("git+") || version_spec.contains("github:") {
        return Ok(None);
    }

    // 如果已经检查过这个包，直接使用缓存结果
    if let Some(latest_version) = checked_packages.get(dep_name) {
        let current_version = extract_version_from_spec(version_spec);
        if current_version != *latest_version
            && !is_version_satisfied(&current_version, latest_version)
        {
            return Ok(Some(OutdatedDependency {
                name: dep_name.to_string(),
                current: current_version,
                latest: latest_version.clone(),
                package: package_name.to_string(),
                dep_type: dep_type.to_string(),
            }));
        }
        return Ok(None);
    }

    // 获取最新版本
    if let Some(latest_version) = get_latest_version(dep_name, verbose)? {
        checked_packages.insert(dep_name.to_string(), latest_version.clone());

        let current_version = extract_version_from_spec(version_spec);
        if current_version != latest_version
            && !is_version_satisfied(&current_version, &latest_version)
        {
            return Ok(Some(OutdatedDependency {
                name: dep_name.to_string(),
                current: current_version,
                latest: latest_version,
                package: package_name.to_string(),
                dep_type: dep_type.to_string(),
            }));
        }
    }

    Ok(None)
}

/// 检查是否应该跳过依赖检查
fn should_skip_dependency(version_spec: &str) -> bool {
    // 跳过工作区依赖
    if version_spec.starts_with("workspace:") {
        return true;
    }

    // 跳过文件路径依赖
    if version_spec.starts_with("file:") || version_spec.starts_with("link:") {
        return true;
    }

    // 跳过 git 依赖
    if version_spec.contains("git+") || version_spec.contains("github:") {
        return true;
    }

    false
}

/// 多线程安全的依赖版本检查
fn check_dependency_version_concurrent(
    dep_name: &str,
    version_spec: &str,
    package_name: &str,
    dep_type: &str,
    checked_packages: &Arc<Mutex<HashMap<String, String>>>,
    verbose: bool,
) -> Result<Option<OutdatedDependency>> {
    // 如果已经检查过这个包，直接使用缓存结果
    if let Ok(cache) = checked_packages.lock() {
        if let Some(latest_version) = cache.get(dep_name) {
            let current_version = extract_version_from_spec(version_spec);
            if current_version != *latest_version
                && !is_version_satisfied(&current_version, latest_version)
            {
                return Ok(Some(OutdatedDependency {
                    name: dep_name.to_string(),
                    current: current_version,
                    latest: latest_version.clone(),
                    package: package_name.to_string(),
                    dep_type: dep_type.to_string(),
                }));
            }
            return Ok(None);
        }
    }

    // 获取最新版本
    if let Some(latest_version) = get_latest_version(dep_name, verbose)? {
        // 更新缓存
        if let Ok(mut cache) = checked_packages.lock() {
            cache.insert(dep_name.to_string(), latest_version.clone());
        }

        let current_version = extract_version_from_spec(version_spec);
        if current_version != latest_version
            && !is_version_satisfied(&current_version, &latest_version)
        {
            return Ok(Some(OutdatedDependency {
                name: dep_name.to_string(),
                current: current_version,
                latest: latest_version,
                package: package_name.to_string(),
                dep_type: dep_type.to_string(),
            }));
        }
    }

    Ok(None)
}

/// 从版本规范中提取版本号
fn extract_version_from_spec(version_spec: &str) -> String {
    // 移除版本前缀符号 (^, ~, >=, etc.)
    let version = version_spec
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches("<=")
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=');

    version.to_string()
}

/// 简单的版本比较（检查当前版本是否满足最新版本）
fn is_version_satisfied(current: &str, latest: &str) -> bool {
    // 简单比较：如果当前版本等于最新版本，则满足
    current == latest
}

/// 使用 npm view 获取包的最新版本
fn get_latest_version(package_name: &str, verbose: bool) -> Result<Option<String>> {
    if verbose {
        Logger::info(tf!("check.outdated.fetching_version", package_name));
    }

    let output = Command::new("npm")
        .args(&["view", package_name, "version", "--json"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // 尝试解析 JSON 响应
                if let Ok(response) = serde_json::from_str::<NpmViewResponse>(&stdout) {
                    return Ok(Some(response.version));
                }

                // 如果 JSON 解析失败，尝试直接解析版本字符串
                let version = stdout.trim().trim_matches('"');
                if !version.is_empty() {
                    return Ok(Some(version.to_string()));
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if verbose {
                    Logger::warn(tf!("check.outdated.npm_error", package_name, stderr));
                }
            }
        }
        Err(e) => {
            if verbose {
                Logger::warn(tf!("check.outdated.npm_command_error", package_name, e));
            }
        }
    }

    Ok(None)
}

/// 打印循环依赖表格
fn print_circular_dependencies_table(circular_dependencies: &[Vec<String>], detail: bool) {
    use crate::utils::constants::icons;

    Logger::info("");
    Logger::info(t!("check.circular.details"));
    Logger::info("───────────────────────────────────────");

    for (index, cycle) in circular_dependencies.iter().enumerate() {
        Logger::info(tf!("check.circular.cycle_header", index + 1));

        if detail {
            // 详细模式：显示完整的循环路径
            for (i, package) in cycle.iter().enumerate() {
                let next_package = &cycle[(i + 1) % cycle.len()];
                Logger::info(tf!(
                    "check.circular.cycle_detail",
                    icons::ARROW,
                    package,
                    next_package
                ));
            }
        } else {
            // 简单模式：只显示涉及的包
            let cycle_str = cycle.join(" → ");
            Logger::info(tf!("check.circular.cycle_simple", cycle_str));
        }

        Logger::info("");
    }

    Logger::info(t!("check.circular.suggestion"));
}

/// 计算最优线程数
fn calculate_optimal_thread_count(dependency_count: usize) -> usize {
    // 获取系统 CPU 核心数
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4); // 默认 4 核

    // 根据依赖数量和 CPU 核心数计算最优线程数
    let optimal_threads = match dependency_count {
        0..=10 => std::cmp::min(dependency_count, 2), // 少量依赖：最多 2 线程
        11..=50 => std::cmp::min(dependency_count / 2, cpu_count), // 中等依赖：每 2 个依赖 1 线程，不超过 CPU 核心数
        51..=200 => std::cmp::min(dependency_count / 4, cpu_count * 2), // 较多依赖：每 4 个依赖 1 线程，最多 CPU 核心数 * 2
        _ => std::cmp::min(dependency_count / 8, cpu_count * 3), // 大量依赖：每 8 个依赖 1 线程，最多 CPU 核心数 * 3
    };

    // 确保至少有 1 个线程，最多不超过依赖数量
    std::cmp::max(1, std::cmp::min(optimal_threads, dependency_count))
}

/// 实时显示发现的过期包
fn print_outdated_package_realtime(dep_name: &str, current: &str, latest: &str, verbose: bool) {
    if verbose {
        // verbose 模式下显示详细信息
        Logger::warn(tf!(
            "check.outdated.found_realtime",
            Colors::info(dep_name),
            current,
            latest
        ));
    }
    // 非 verbose 模式下不显示实时信息，避免与表格输出重复
}

fn print_outdated_dependencies_table(
    outdated_deps: &[OutdatedDependency],
    detail: bool,
) -> Result<()> {
    use crate::utils::colors::Colors;
    use std::collections::BTreeMap;

    Logger::info("");
    Logger::info(t!("check.outdated.details"));
    Logger::info("───────────────────────────────────────");

    if detail {
        // 详细模式：按包分组显示
        let mut packages: BTreeMap<String, Vec<&OutdatedDependency>> = BTreeMap::new();
        for dep in outdated_deps {
            packages.entry(dep.package.clone()).or_default().push(dep);
        }

        for (package_name, deps) in packages {
            Logger::info(tf!("check.outdated.package_header", package_name));

            // 按依赖名称去重
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
    } else {
        // 简单模式：去重显示，只显示包名和版本
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
            // 显示依赖名称和版本信息
            Logger::info(tf!(
                "check.outdated.dep_simple_single",
                Colors::info(&format!("[{}]", dep.name)),
                dep.current,
                dep.latest
            ));

            // 逐行显示使用该依赖的包
            for package in &packages {
                Logger::info(format!("    {}", package));
            }
        }
        Logger::info("");
    }

    // 根据配置的包管理器显示建议
    let package_manager = Config::get_package_manager();
    let suggestion = match package_manager.as_str() {
        "pnpm" => t!("check.outdated.suggestion_pnpm"),
        "yarn" => t!("check.outdated.suggestion_yarn"),
        "npm" | _ => t!("check.outdated.suggestion_npm"),
    };
    Logger::info(suggestion);

    Ok(())
}

/// 收集版本冲突
pub fn collect_version_conflicts(
    package_files: &[std::path::PathBuf],
    verbose: bool,
) -> Result<Vec<VersionConflict>> {
    use std::collections::{BTreeMap, HashMap};
    use std::fs;

    // 按依赖名称分组收集所有使用情况
    let mut dependency_usages: BTreeMap<String, Vec<ConflictUsage>> = BTreeMap::new();

    for package_file in package_files {
        if verbose {
            Logger::info(tf!(
                "check.versions.processing_package",
                package_file.display()
            ));
        }

        let content = fs::read_to_string(package_file)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        let package_name = package_json["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // 处理不同类型的依赖
        let dep_types = ["dependencies", "devDependencies", "peerDependencies"];

        for dep_type in &dep_types {
            if let Some(deps) = package_json[dep_type].as_object() {
                for (dep_name, version_value) in deps {
                    let version_spec = version_value.as_str().unwrap_or("").to_string();

                    // 跳过特殊依赖
                    if should_skip_dependency(&version_spec) {
                        continue;
                    }

                    let resolved_version = extract_version_from_spec(&version_spec);

                    let usage = ConflictUsage {
                        package: package_name.clone(),
                        version_spec: version_spec.clone(),
                        resolved_version,
                        dep_type: dep_type.to_string(),
                    };

                    dependency_usages
                        .entry(dep_name.clone())
                        .or_default()
                        .push(usage);
                }
            }
        }
    }

    // 检查每个依赖是否有版本冲突
    let mut conflicts = Vec::new();

    for (dep_name, usages) in dependency_usages {
        if usages.len() < 2 {
            continue; // 只有一个使用者，不会有冲突
        }

        // 检查是否存在版本冲突
        let mut unique_versions: HashMap<String, Vec<&ConflictUsage>> = HashMap::new();
        for usage in &usages {
            unique_versions
                .entry(usage.resolved_version.clone())
                .or_default()
                .push(usage);
        }

        if unique_versions.len() > 1 {
            // 存在版本冲突
            let recommended_version = calculate_recommended_version(&usages);

            let conflict = VersionConflict {
                name: dep_name,
                conflicts: usages,
                recommended_version,
            };

            conflicts.push(conflict);
        }
    }

    if verbose {
        Logger::info(tf!("check.versions.conflicts_found", conflicts.len()));
    }

    Ok(conflicts)
}

/// 计算推荐的统一版本（选择最高版本）
fn calculate_recommended_version(usages: &[ConflictUsage]) -> String {
    let mut versions: Vec<String> = usages
        .iter()
        .map(|usage| usage.resolved_version.clone())
        .collect();

    // 简单的字符串排序，实际项目中可能需要更复杂的语义版本比较
    versions.sort();
    versions.dedup();

    // 返回最高版本（字符串排序的最后一个）
    // 注意：这是一个简化的实现，实际应该使用 semver 库进行正确的版本比较
    versions
        .last()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string())
}

/// 打印版本冲突表格
fn print_version_conflicts_table(conflicts: &[VersionConflict], detail: bool) -> Result<()> {
    use crate::utils::colors::Colors;

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
            // 详细模式：显示所有使用情况
            for usage in &conflict.conflicts {
                Logger::info(tf!(
                    "check.versions.usage_detail",
                    usage.package,
                    usage.version_spec,
                    usage.resolved_version,
                    usage.dep_type
                ));
            }
        } else {
            // 简单模式：按版本分组显示
            use std::collections::HashMap;
            let mut version_groups: HashMap<String, Vec<&ConflictUsage>> = HashMap::new();

            for usage in &conflict.conflicts {
                version_groups
                    .entry(usage.resolved_version.clone())
                    .or_default()
                    .push(usage);
            }

            for (version, usages) in version_groups {
                let packages: Vec<String> = usages.iter().map(|u| u.package.clone()).collect();
                Logger::info(tf!(
                    "check.versions.version_group",
                    version,
                    packages.join(", ")
                ));
            }
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
