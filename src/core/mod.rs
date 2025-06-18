// ============================================================================
// MonoX - Core 核心模块
// ============================================================================
//
// 文件: src/core/mod.rs
// 职责: 核心业务逻辑模块入口和导出
// 边界:
//   - ✅ 核心子模块导出
//   - ✅ 常用类型重新导出
//   - ✅ 模块间接口定义
//   - ❌ 不应包含具体业务实现
//   - ❌ 不应包含 CLI 相关逻辑
//   - ❌ 不应包含 UI 相关逻辑
//   - ❌ 不应包含工具函数实现
//
// ============================================================================

pub mod analyzer;
pub mod cache;
pub mod executor;
pub mod scheduler;

// 重新导出常用类型
pub use analyzer::DependencyAnalyzer;
pub use executor::TaskExecutor;
pub use scheduler::{
    AsyncTaskScheduler, ExecutionSummary, SchedulerConfig, TaskResult as SchedulerTaskResult,
};
