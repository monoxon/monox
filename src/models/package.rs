// ============================================================================
// MonoX - 包数据模型
// ============================================================================
//
// 文件: src/models/package.rs
// 职责: 包信息和分析结果数据结构定义
// 边界:
//   - ✅ 包信息数据结构定义
//   - ✅ 分析结果数据结构定义
//   - ✅ 数据序列化/反序列化
//   - ✅ 基础数据操作方法
//   - ❌ 不应包含包扫描逻辑
//   - ❌ 不应包含依赖分析算法
//   - ❌ 不应包含文件解析逻辑
//   - ❌ 不应包含业务规则验证
//
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// 工作区包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePackage {
    /// 包名
    pub name: String,
    /// 包的相对路径
    pub folder: PathBuf,
    /// 包的绝对路径
    pub absolute_path: PathBuf,
    /// 版本
    pub version: String,
    /// 所有依赖（包括 dependencies, devDependencies, peerDependencies）
    pub dependencies: HashMap<String, String>,
    /// 仅工作区内的依赖
    pub workspace_dependencies: HashSet<String>,
    /// 构建脚本
    pub scripts: HashMap<String, String>,
}

/// package.json 文件结构（用于解析）
#[derive(Debug, Clone, Deserialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
}

/// 依赖分析结果
#[derive(Debug, Clone, Serialize)]
pub struct DependencyAnalysisResult {
    /// 所有工作区包
    pub packages: Vec<WorkspacePackage>,
    /// 按依赖顺序分组的构建阶段
    pub stages: Vec<Vec<WorkspacePackage>>,
    /// 循环依赖（如果存在）
    pub circular_dependencies: Vec<Vec<String>>,
    /// 分析统计信息
    pub statistics: AnalysisStatistics,
}

/// 分析统计信息
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisStatistics {
    /// 总包数
    pub total_packages: usize,
    /// 总阶段数
    pub total_stages: usize,
    /// 有工作区依赖的包数量
    pub packages_with_workspace_deps: usize,
    /// 循环依赖数量
    pub circular_dependency_count: usize,
    /// 分析耗时（毫秒）
    pub analysis_duration_ms: u64,
}

impl WorkspacePackage {
    /// 创建新的工作区包
    pub fn new(
        name: String,
        folder: PathBuf,
        absolute_path: PathBuf,
        version: String,
        dependencies: HashMap<String, String>,
        scripts: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            folder,
            absolute_path,
            version,
            dependencies,
            workspace_dependencies: HashSet::new(),
            scripts,
        }
    }

    /// 检查是否有特定的依赖
    pub fn has_dependency(&self, dep_name: &str) -> bool {
        self.dependencies.contains_key(dep_name)
    }

    /// 检查是否有工作区依赖
    pub fn has_workspace_dependencies(&self) -> bool {
        !self.workspace_dependencies.is_empty()
    }

    /// 添加工作区依赖
    pub fn add_workspace_dependency(&mut self, dep_name: String) {
        self.workspace_dependencies.insert(dep_name);
    }
}

impl PackageJson {
    /// 获取所有依赖的合并结果
    pub fn get_all_dependencies(&self) -> HashMap<String, String> {
        let mut all_deps = HashMap::new();

        // 合并所有类型的依赖
        all_deps.extend(self.dependencies.clone());
        all_deps.extend(self.dev_dependencies.clone());
        all_deps.extend(self.peer_dependencies.clone());

        all_deps
    }

    /// 获取包名，如果没有则使用目录名
    pub fn get_name(&self, fallback_name: &str) -> String {
        self.name.clone().unwrap_or_else(|| fallback_name.to_string())
    }

    /// 获取版本，如果没有则使用默认版本
    pub fn get_version(&self) -> String {
        self.version.clone().unwrap_or_else(|| "0.0.0".to_string())
    }
    pub fn from_file(package_path: &str) -> Self {
        let package_json = fs::read_to_string(&format!("{}/package.json", package_path)).unwrap();
        serde_json::from_str(&package_json).unwrap()
    }

    pub fn has_script(&self, script_name: &str) -> bool {
        self.scripts.contains_key(script_name)
    }
}

impl Default for AnalysisStatistics {
    fn default() -> Self {
        Self {
            total_packages: 0,
            total_stages: 0,
            packages_with_workspace_deps: 0,
            circular_dependency_count: 0,
            analysis_duration_ms: 0,
        }
    }
}
