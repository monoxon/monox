// ============================================================================
// MonoX - 健康检查器
// ============================================================================
//
// 文件: src/core/checker.rs
// 职责: 工作区健康检查核心逻辑
// 边界:
//   - ✅ 循环依赖检测和分析
//   - ✅ 版本冲突检测和分析
//   - ✅ 过期依赖检测和分析
//   - ✅ package.json 解析和依赖收集
//   - ✅ 异步任务调度和执行
//   - ❌ 不应包含CLI参数处理
//   - ❌ 不应包含输出格式化
//   - ❌ 不应包含用户交互
//   - ❌ 不应包含国际化文本
//
// ============================================================================

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::sync::{Arc, Mutex};

use crate::core::analyzer::DependencyAnalyzer;
use crate::core::scheduler::{AsyncTaskScheduler, SchedulerConfig};
use crate::models::config::Config;

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
    used_by: Vec<(String, String)>,
}

/// npm view 命令的响应结构
#[derive(Debug, Deserialize)]
struct NpmViewResponse {
    version: String,
}

/// 进度回调函数类型
pub type ProgressCallback = Arc<dyn Fn(usize, usize) + Send + Sync>;

/// 健康检查器
pub struct HealthChecker {
    workspace_root: std::path::PathBuf,
    verbose: bool,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(workspace_root: std::path::PathBuf) -> Self {
        Self {
            workspace_root,
            verbose: false,
        }
    }

    /// 设置详细模式
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 检查循环依赖
    pub fn check_circular_dependencies(&self) -> Result<Vec<Vec<String>>> {
        let mut analyzer =
            DependencyAnalyzer::new(self.workspace_root.clone()).with_verbose(self.verbose);
        let result = analyzer.analyze_workspace()?;
        Ok(result.circular_dependencies)
    }

    /// 检查版本冲突
    pub fn check_version_conflicts(&self) -> Result<Vec<VersionConflict>> {
        let package_files = self.collect_package_files()?;
        if package_files.is_empty() {
            return Ok(Vec::new());
        }
        self.collect_version_conflicts(&package_files)
    }

    /// 检查过期依赖
    pub async fn check_outdated_dependencies(&self) -> Result<Vec<OutdatedDependency>> {
        let (outdated_deps, _) = self.check_outdated_dependencies_with_progress(None).await?;
        Ok(outdated_deps)
    }

    /// 检查过期依赖（带进度回调）
    /// 返回 (过期依赖列表, 总检测依赖数量)
    pub async fn check_outdated_dependencies_with_progress(
        &self,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<(Vec<OutdatedDependency>, usize)> {
        let package_files = self.collect_package_files()?;
        if package_files.is_empty() {
            return Ok((Vec::new(), 0));
        }

        let unique_dependencies = self.collect_unique_dependencies(&package_files)?;
        if unique_dependencies.is_empty() {
            return Ok((Vec::new(), 0));
        }

        // 获取固定的总依赖数量，并创建包装的进度回调
        let total_deps = unique_dependencies.len();
        let wrapped_callback = progress_callback.map(|callback| {
            Arc::new(move |completed: usize, _dynamic_total: usize| {
                // 忽略动态总数，使用固定的总数
                callback(completed, total_deps);
            }) as ProgressCallback
        });

        let outdated_deps = self
            .check_outdated_with_scheduler(unique_dependencies, wrapped_callback)
            .await?;

        Ok((outdated_deps, total_deps))
    }
}

// ============================================================================
// 文件扫描和依赖收集
// ============================================================================

impl HealthChecker {
    /// 扫描目录收集 package.json 文件
    fn scan_directory_for_packages(
        &self,
        dir: &std::path::Path,
        package_files: &mut Vec<std::path::PathBuf>,
    ) -> Result<()> {
        let entries = fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(dir.parent().unwrap_or(dir))
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            if Config::should_ignore_path(&relative_path).unwrap_or(false) {
                continue;
            }

            if path.is_dir() {
                self.scan_directory_for_packages(&path, package_files)?;
            } else if path.file_name() == Some(std::ffi::OsStr::new("package.json")) {
                package_files.push(path);
            }
        }

        Ok(())
    }

    /// 收集所有未被忽略的 package.json 文件
    fn collect_package_files(&self) -> Result<Vec<std::path::PathBuf>> {
        let mut package_files = Vec::new();
        self.scan_directory_for_packages(&self.workspace_root, &mut package_files)?;
        Ok(package_files)
    }

    /// 收集并去重所有依赖
    fn collect_unique_dependencies(
        &self,
        package_files: &[std::path::PathBuf],
    ) -> Result<BTreeMap<String, DependencyInfo>> {
        let mut unique_dependencies: BTreeMap<String, DependencyInfo> = BTreeMap::new();

        for package_file in package_files {
            let package_json = parse_package_json(package_file)?;
            let package_name = package_json["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            process_package_dependencies(&package_json, &package_name, &mut unique_dependencies);
        }

        Ok(unique_dependencies)
    }
}

// ============================================================================
// 过期依赖检查
// ============================================================================

impl HealthChecker {
    /// 使用调度器检查过期依赖
    async fn check_outdated_with_scheduler(
        &self,
        unique_dependencies: BTreeMap<String, DependencyInfo>,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Vec<OutdatedDependency>> {
        let total_deps = unique_dependencies.len();
        let outdated_deps = Arc::new(Mutex::new(Vec::new()));
        let found_packages = Arc::new(Mutex::new(std::collections::HashSet::<String>::new()));

        // 创建调度器配置
        let config = SchedulerConfig {
            max_concurrency: calculate_optimal_thread_count(total_deps),
            timeout: Some(std::time::Duration::from_secs(30)),
            fail_fast: false,
            verbose: self.verbose,
            progress_callback,
            task_completed_callback: None,
        };

        let scheduler = AsyncTaskScheduler::new(config);
        let tasks = create_outdated_check_tasks(
            unique_dependencies,
            outdated_deps.clone(),
            found_packages,
            self.verbose,
        );
        let _results = scheduler.execute_batch(tasks).await;

        let result = outdated_deps.lock().unwrap().clone();
        Ok(result)
    }
}

// ============================================================================
// 版本冲突检查
// ============================================================================

impl HealthChecker {
    /// 收集版本冲突
    fn collect_version_conflicts(
        &self,
        package_files: &[std::path::PathBuf],
    ) -> Result<Vec<VersionConflict>> {
        let mut dependency_usages: BTreeMap<String, Vec<ConflictUsage>> = BTreeMap::new();

        // 收集所有依赖使用情况
        for package_file in package_files {
            let package_json = parse_package_json(package_file)?;
            let package_name = package_json["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            collect_dependency_usages(&package_json, &package_name, &mut dependency_usages);
        }

        // 检查版本冲突
        let conflicts = find_version_conflicts(dependency_usages);
        Ok(conflicts)
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 依赖类型常量
const DEP_TYPES: &[&str] = &["dependencies", "devDependencies", "peerDependencies"];

/// 解析 package.json 文件
fn parse_package_json(package_file: &std::path::PathBuf) -> Result<serde_json::Value> {
    let content = fs::read_to_string(package_file)?;
    Ok(serde_json::from_str(&content)?)
}

/// 处理单个包的依赖
fn process_package_dependencies(
    package_json: &serde_json::Value,
    package_name: &str,
    unique_dependencies: &mut BTreeMap<String, DependencyInfo>,
) {
    for dep_type in DEP_TYPES {
        if let Some(deps) = package_json[dep_type].as_object() {
            for (dep_name, version_value) in deps {
                let version_spec = version_value.as_str().unwrap_or("").to_string();

                if should_skip_dependency(&version_spec) {
                    continue;
                }

                add_or_update_dependency(
                    unique_dependencies,
                    dep_name,
                    &version_spec,
                    package_name,
                    dep_type,
                );
            }
        }
    }
}

/// 添加或更新依赖信息
fn add_or_update_dependency(
    unique_dependencies: &mut BTreeMap<String, DependencyInfo>,
    dep_name: &str,
    version_spec: &str,
    package_name: &str,
    dep_type: &str,
) {
    unique_dependencies
        .entry(dep_name.to_string())
        .and_modify(|dep_info| {
            dep_info
                .used_by
                .push((package_name.to_string(), dep_type.to_string()));
        })
        .or_insert_with(|| DependencyInfo {
            name: dep_name.to_string(),
            version_spec: version_spec.to_string(),
            used_by: vec![(package_name.to_string(), dep_type.to_string())],
        });
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

/// 简单的版本比较
fn is_version_satisfied(current: &str, latest: &str) -> bool {
    current == latest
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
    match serde_json::from_str::<NpmViewResponse>(trimmed) {
        Ok(response) => Ok(Some(response.version)),
        Err(_) => {
            let version = trimmed.trim_matches('"');
            Ok(Some(version.to_string()))
        }
    }
}

/// 计算最优线程数
fn calculate_optimal_thread_count(dependency_count: usize) -> usize {
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let optimal_threads = match dependency_count {
        0..=10 => std::cmp::min(dependency_count, 2),
        11..=50 => std::cmp::min(dependency_count / 2, cpu_count),
        51..=200 => std::cmp::min(dependency_count / 4, cpu_count * 2),
        _ => std::cmp::min(dependency_count / 8, cpu_count * 3),
    };

    std::cmp::max(1, std::cmp::min(optimal_threads, dependency_count))
}

/// 创建过期检查任务
fn create_outdated_check_tasks(
    unique_dependencies: BTreeMap<String, DependencyInfo>,
    outdated_deps: Arc<Mutex<Vec<OutdatedDependency>>>,
    found_packages: Arc<Mutex<std::collections::HashSet<String>>>,
    verbose: bool,
) -> Vec<(
    String,
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
)> {
    unique_dependencies
        .into_iter()
        .map(|(dep_name, dep_info)| {
            let outdated_deps = Arc::clone(&outdated_deps);
            let found_packages = Arc::clone(&found_packages);
            let task_name = dep_name.clone();

            let task_future: std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<()>> + Send>,
            > = Box::pin(async move {
                process_dependency_version(
                    dep_name,
                    dep_info,
                    outdated_deps,
                    found_packages,
                    verbose,
                )
                .await
            });

            (task_name, task_future)
        })
        .collect()
}

/// 处理单个依赖的版本检查
async fn process_dependency_version(
    dep_name: String,
    dep_info: DependencyInfo,
    outdated_deps: Arc<Mutex<Vec<OutdatedDependency>>>,
    found_packages: Arc<Mutex<std::collections::HashSet<String>>>,
    _verbose: bool,
) -> Result<()> {
    let latest_version = match get_latest_version_async(&dep_name).await? {
        Some(version) => version,
        None => return Ok(()),
    };

    let current_version = extract_version_from_spec(&dep_info.version_spec);

    if current_version == latest_version || is_version_satisfied(&current_version, &latest_version)
    {
        return Ok(());
    }

    // 记录发现的过期包
    let _is_new_package = {
        let mut found_set = found_packages.lock().unwrap();
        found_set.insert(dep_name.clone())
    };

    // 为每个使用该依赖的包创建记录
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

    Ok(())
}

/// 收集依赖使用情况
fn collect_dependency_usages(
    package_json: &serde_json::Value,
    package_name: &str,
    dependency_usages: &mut BTreeMap<String, Vec<ConflictUsage>>,
) {
    for dep_type in DEP_TYPES {
        if let Some(deps) = package_json[dep_type].as_object() {
            for (dep_name, version_value) in deps {
                let version_spec = version_value.as_str().unwrap_or("").to_string();

                if should_skip_dependency(&version_spec) {
                    continue;
                }

                let resolved_version = extract_version_from_spec(&version_spec);
                let usage = ConflictUsage {
                    package: package_name.to_string(),
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

/// 查找版本冲突
fn find_version_conflicts(
    dependency_usages: BTreeMap<String, Vec<ConflictUsage>>,
) -> Vec<VersionConflict> {
    let mut conflicts = Vec::new();

    for (dep_name, usages) in dependency_usages {
        if usages.len() < 2 {
            continue;
        }

        // 检查是否存在版本冲突
        let unique_versions: HashMap<String, Vec<&ConflictUsage>> = group_by_version(&usages);

        if unique_versions.len() > 1 {
            let recommended_version = calculate_recommended_version(&usages);
            conflicts.push(VersionConflict {
                name: dep_name,
                conflicts: usages,
                recommended_version,
            });
        }
    }

    conflicts
}

/// 按版本分组
fn group_by_version(usages: &[ConflictUsage]) -> HashMap<String, Vec<&ConflictUsage>> {
    let mut unique_versions: HashMap<String, Vec<&ConflictUsage>> = HashMap::new();
    for usage in usages {
        unique_versions
            .entry(usage.resolved_version.clone())
            .or_default()
            .push(usage);
    }
    unique_versions
}

/// 计算推荐的统一版本
fn calculate_recommended_version(usages: &[ConflictUsage]) -> String {
    let mut versions: Vec<String> = usages
        .iter()
        .map(|usage| usage.resolved_version.clone())
        .collect();

    versions.sort();
    versions.dedup();

    versions
        .last()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string())
}
