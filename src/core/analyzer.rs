// ============================================================================
// MonoX - 依赖分析器
// ============================================================================
//
// 文件: src/core/analyzer.rs
// 职责: 工作区依赖关系分析核心逻辑
// 边界:
//   - ✅ 工作区包扫描和解析
//   - ✅ 依赖关系图构建和分析
//   - ✅ 循环依赖检测
//   - ✅ 构建阶段计算
//   - ✅ 配置文件加载和应用
//   - ✅ 用户交互和进度显示
//   - ✅ 运行结果格式化输出
//   - ❌ 不应包含 CLI 参数处理
//   - ❌ 不应包含具体构建执行逻辑
//
// 算法设计:
// 1. 扫描 monorepo 目录下所有 package.json 文件
// 2. 解析包信息和依赖关系，构建工作区包字典
// 3. 使用拓扑排序计算构建阶段
// 4. 使用 Tarjan 算法检测强连通分量（循环依赖）
//
// ============================================================================

// 依赖分析器

use anyhow::{Context, Result};
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

use crate::models::config::Config;
use crate::models::package::{
    AnalysisStatistics, DependencyAnalysisResult, PackageJson, WorkspacePackage,
};
use crate::utils::logger::Logger;
use crate::{t, tf};

/// 依赖分析器
pub struct DependencyAnalyzer {
    /// 工作区根目录
    workspace_root: PathBuf,
    /// 是否启用详细日志
    verbose: bool,
}

impl DependencyAnalyzer {
    /// 创建新的依赖分析器
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            verbose: false,
        }
    }

    /// 启用详细日志
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 分析工作区依赖关系
    pub fn analyze(&mut self) -> Result<DependencyAnalysisResult> {
        let start_time = Instant::now();

        if self.verbose {
            Logger::info(tf!(
                "analyze.scanning_workspace",
                self.workspace_root.display()
            ));
        }

        // 1. 扫描所有包
        let mut packages = self.scan_workspace_packages()?;

        if self.verbose {
            Logger::info(tf!("analyze.found_packages", packages.len()));
        }

        // 2. 分析工作区依赖关系
        self.analyze_workspace_dependencies(&mut packages);

        // 3. 构建依赖图
        let (graph, node_map) = self.build_dependency_graph(&packages)?;

        // 4. 检测循环依赖
        let circular_dependencies = self.detect_circular_dependencies(&graph, &node_map);

        // 5. 计算构建阶段
        let stages = if circular_dependencies.is_empty() {
            self.calculate_build_stages(&packages)
        } else {
            if self.verbose {
                Logger::info(t!("analyze.circular_detected"));
            }
            Vec::new()
        };

        let analysis_duration = start_time.elapsed().as_millis() as u64;

        // 6. 生成统计信息
        let statistics = AnalysisStatistics {
            total_packages: packages.len(),
            total_stages: stages.len(),
            packages_with_workspace_deps: packages
                .iter()
                .filter(|p| p.has_workspace_dependencies())
                .count(),
            circular_dependency_count: circular_dependencies.len(),
            analysis_duration_ms: analysis_duration,
        };

        if self.verbose {
            Logger::info(tf!("analyze.completed", analysis_duration, stages.len()));
        }

        Ok(DependencyAnalysisResult {
            packages,
            stages,
            circular_dependencies,
            statistics,
        })
    }

    /// 扫描工作区中的所有包
    fn scan_workspace_packages(&self) -> Result<Vec<WorkspacePackage>> {
        let mut packages = Vec::new();

        // 使用 walkdir 遍历目录
        for entry in WalkDir::new(&self.workspace_root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let relative_path = e
                    .path()
                    .strip_prefix(&self.workspace_root)
                    .unwrap_or(e.path())
                    .to_string_lossy();

                // 检查忽略模式（完全跳过，不进入子目录）
                match Config::should_ignore_path(&relative_path) {
                    Ok(should_ignore) => !should_ignore,
                    Err(_) => !relative_path.contains("node_modules"), // 配置错误时的后备逻辑
                }
            })
        {
            let entry = entry.context(t!("error.walk_directory"))?;

            // 查找 package.json 文件
            if entry.file_name() == "package.json" {
                let package_path = entry.path();

                // 排除根目录的 package.json
                if package_path.parent() == Some(&self.workspace_root) {
                    if self.verbose {
                        Logger::info(tf!("analyze.skip_root_package", package_path.display()));
                    }
                    continue;
                }

                if let Ok(package) = self.parse_package_json(package_path) {
                    packages.push(package);
                } else if self.verbose {
                    Logger::info(tf!("analyze.skip_invalid_package", package_path.display()));
                }
            }
        }

        if packages.is_empty() {
            anyhow::bail!(t!("error.no_packages_found"));
        }

        Ok(packages)
    }

    /// 解析单个 package.json 文件
    fn parse_package_json(&self, package_json_path: &Path) -> Result<WorkspacePackage> {
        let content = fs::read_to_string(package_json_path).with_context(|| {
            tf!("error.read_package_json", package_json_path.display()).to_string()
        })?;

        let package_json: PackageJson = serde_json::from_str(&content).with_context(|| {
            tf!("error.parse_package_json", package_json_path.display()).to_string()
        })?;

        let package_dir = package_json_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!(t!("error.get_package_dir")))?;

        // 计算相对路径
        let relative_path = package_dir
            .strip_prefix(&self.workspace_root)
            .unwrap_or(package_dir)
            .to_path_buf();

        // 使用目录名作为后备包名
        let fallback_name = package_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let package = WorkspacePackage::new(
            package_json.get_name(fallback_name),
            relative_path,
            package_dir.to_path_buf(),
            package_json.get_version(),
            package_json.get_all_dependencies(),
            package_json.scripts,
        );

        Ok(package)
    }

    /// 分析工作区内的依赖关系
    fn analyze_workspace_dependencies(&self, packages: &mut [WorkspacePackage]) {
        // 创建包名到索引的映射
        let package_names: HashSet<String> = packages.iter().map(|p| p.name.clone()).collect();

        // 为每个包标记工作区依赖
        for package in packages.iter_mut() {
            let workspace_deps: Vec<String> = package
                .dependencies
                .keys()
                .filter(|dep_name| package_names.contains(*dep_name))
                .cloned()
                .collect();

            for dep_name in workspace_deps {
                package.add_workspace_dependency(dep_name);
            }
        }
    }

    /// 构建依赖图
    fn build_dependency_graph(
        &self,
        packages: &[WorkspacePackage],
    ) -> Result<(DiGraph<String, ()>, HashMap<String, NodeIndex>)> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        // 添加所有节点
        for package in packages {
            let node_idx = graph.add_node(package.name.clone());
            node_map.insert(package.name.clone(), node_idx);
        }

        // 添加依赖边
        for package in packages {
            let package_node = node_map[&package.name];

            for dep_name in &package.workspace_dependencies {
                if let Some(&dep_node) = node_map.get(dep_name) {
                    // 添加从被依赖包到依赖包的边（dep_name -> package.name）
                    graph.add_edge(dep_node, package_node, ());
                }
            }
        }

        Ok((graph, node_map))
    }

    /// 检测循环依赖
    fn detect_circular_dependencies(
        &self,
        graph: &DiGraph<String, ()>,
        _node_map: &HashMap<String, NodeIndex>,
    ) -> Vec<Vec<String>> {
        let mut circular_deps = Vec::new();

        // 使用 Tarjan's 强连通分量算法
        let sccs = tarjan_scc(graph);

        for scc in sccs {
            // 只有包含多个节点的强连通分量才是循环依赖
            if scc.len() > 1 {
                let cycle: Vec<String> = scc
                    .iter()
                    .map(|&node_idx| graph[node_idx].clone())
                    .collect();
                circular_deps.push(cycle);
            }
        }

        if self.verbose && !circular_deps.is_empty() {
            Logger::info(tf!("analyze.circular_found", circular_deps.len()));
            for (i, cycle) in circular_deps.iter().enumerate() {
                Logger::info(tf!("analyze.circular_detail", i + 1, cycle.join(" -> ")));
            }
        }

        circular_deps
    }

    /// 计算构建阶段（基于拓扑排序）
    fn calculate_build_stages(&self, packages: &[WorkspacePackage]) -> Vec<Vec<WorkspacePackage>> {
        let mut stages = Vec::new();

        // 创建包名到包的映射
        let package_map: HashMap<String, WorkspacePackage> = packages
            .iter()
            .map(|p| (p.name.clone(), p.clone()))
            .collect();

        // 未分配到阶段的包
        let mut unstaged_packages: HashSet<String> =
            packages.iter().map(|p| p.name.clone()).collect();

        while !unstaged_packages.is_empty() {
            let mut current_stage = Vec::new();
            let mut packages_to_remove = Vec::new();

            // 寻找可以在当前阶段构建的包
            for package_name in &unstaged_packages {
                let package = &package_map[package_name];

                // 检查是否所有工作区依赖都已在前面的阶段中
                let can_build_now = package
                    .workspace_dependencies
                    .iter()
                    .all(|dep| !unstaged_packages.contains(dep));

                if can_build_now {
                    current_stage.push(package.clone());
                    packages_to_remove.push(package_name.clone());
                }
            }

            if current_stage.is_empty() {
                // 如果没有包可以构建，说明存在循环依赖
                if self.verbose {
                    let remaining_packages: Vec<String> =
                        unstaged_packages.iter().cloned().collect();
                    Logger::info(tf!(
                        "analyze.circular_warning",
                        remaining_packages.join(", ")
                    ));
                }
                break;
            }

            // 从未分配列表中移除当前阶段的包
            for package_name in packages_to_remove {
                unstaged_packages.remove(&package_name);
            }

            if self.verbose {
                Logger::info(tf!(
                    "analyze.stage_info",
                    stages.len() + 1,
                    current_stage.len(),
                    current_stage
                        .iter()
                        .map(|p| p.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }

            stages.push(current_stage);
        }

        stages
    }

    /// 扫描单个包（用于单包分析）
    pub fn scan_single_package(&self, package_path: &Path) -> Result<WorkspacePackage> {
        let package_json_path = package_path.join("package.json");

        if !package_json_path.exists() {
            anyhow::bail!("指定路径不包含 package.json: {}", package_path.display());
        }

        self.parse_package_json(&package_json_path)
    }

    /// 分析单个包（通过包名）
    pub fn analyze_single_package(
        &mut self,
        package_name: &str,
    ) -> Result<DependencyAnalysisResult> {
        let start_time = Instant::now();

        if self.verbose {
            Logger::info(tf!("analyze.single_package_start", package_name));
        }

        // 1. 执行完整的工作区分析以获得正确的依赖关系
        let full_result = self.analyze()?;

        // 2. 查找目标包
        let target_package = full_result
            .packages
            .iter()
            .find(|p| p.name == package_name)
            .ok_or_else(|| anyhow::anyhow!(tf!("error.package_not_found", package_name)))?
            .clone();

        if self.verbose {
            Logger::info(tf!(
                "analyze.single_package_found",
                package_name,
                target_package.folder.display()
            ));
        }

        // 3. 获取与目标包相关的所有包（包括依赖链）
        let related_packages = self.get_related_packages(&target_package, &full_result.packages);

        // 4. 重新计算相关包的构建阶段
        let stages = if full_result.circular_dependencies.is_empty() {
            self.calculate_build_stages(&related_packages)
        } else {
            // 检查循环依赖是否涉及目标包
            let target_in_cycle = full_result
                .circular_dependencies
                .iter()
                .any(|cycle| cycle.contains(&target_package.name));

            if target_in_cycle {
                if self.verbose {
                    Logger::info(t!("analyze.circular_detected"));
                }
                Vec::new()
            } else {
                self.calculate_build_stages(&related_packages)
            }
        };

        let analysis_duration = start_time.elapsed().as_millis() as u64;

        // 5. 生成统计信息
        let statistics = AnalysisStatistics {
            total_packages: 1, // 单包分析只统计目标包
            total_stages: stages.len(),
            packages_with_workspace_deps: if target_package.has_workspace_dependencies() {
                1
            } else {
                0
            },
            circular_dependency_count: full_result.circular_dependencies.len(),
            analysis_duration_ms: analysis_duration,
        };

        if self.verbose {
            Logger::info(tf!(
                "analyze.single_package_completed",
                package_name,
                analysis_duration
            ));
        }

        // 6. 返回结果（只包含目标包，但保留完整的依赖上下文）
        Ok(DependencyAnalysisResult {
            packages: vec![target_package],
            stages,
            circular_dependencies: full_result.circular_dependencies,
            statistics,
        })
    }

    /// 获取与目标包相关的所有包（只包含目标包及其依赖链）
    fn get_related_packages(
        &self,
        target_package: &WorkspacePackage,
        all_packages: &[WorkspacePackage],
    ) -> Vec<WorkspacePackage> {
        let mut related_packages = Vec::new();
        let mut package_names = HashSet::new();

        // 1. 递归添加目标包的所有依赖（构建目标包需要的完整依赖链）
        self.add_dependencies_for_single_package(
            target_package,
            all_packages,
            &mut related_packages,
            &mut package_names,
        );

        // 2. 添加目标包本身
        if !package_names.contains(&target_package.name) {
            related_packages.push(target_package.clone());
            package_names.insert(target_package.name.clone());
        }

        related_packages
    }

    /// 递归添加单包分析所需的依赖包
    fn add_dependencies_for_single_package(
        &self,
        package: &WorkspacePackage,
        all_packages: &[WorkspacePackage],
        related_packages: &mut Vec<WorkspacePackage>,
        visited: &mut HashSet<String>,
    ) {
        for dep_name in &package.workspace_dependencies {
            if !visited.contains(dep_name) {
                if let Some(dep_package) = all_packages.iter().find(|p| p.name == *dep_name) {
                    visited.insert(dep_name.clone());
                    related_packages.push(dep_package.clone());
                    // 递归添加依赖的依赖（构建目标包需要完整的依赖链）
                    self.add_dependencies_for_single_package(
                        dep_package,
                        all_packages,
                        related_packages,
                        visited,
                    );
                }
            }
        }
    }

    /// 获取工作区根目录
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }
}
