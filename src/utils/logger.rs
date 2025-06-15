// ============================================================================
// MonoX - 日志工具
// ============================================================================
//
// 文件: src/utils/logger.rs
// 职责: 日志输出和格式化工具
// 边界:
//   - ✅ 日志级别管理
//   - ✅ 日志格式化输出
//   - ✅ 日志初始化配置
//   - ✅ 控制台输出控制
//   - ❌ 不应包含业务逻辑
//   - ❌ 不应包含文件日志写入
//   - ❌ 不应包含日志内容生成
//   - ❌ 不应包含特定领域逻辑
//
// ============================================================================

use super::colors::Colors;

/// 简单的日志工具
pub struct Logger;

impl Logger {
    pub fn info<S: AsRef<str>>(msg: S) {
        println!("{} {}", Colors::info("[MONOX]"), msg.as_ref());
    }

    pub fn warn<S: AsRef<str>>(msg: S) {
        println!("{} {}", Colors::warn("[WARN]"), msg.as_ref());
    }

    pub fn error<S: AsRef<str>>(msg: S) {
        eprintln!("{} {}", Colors::error("[ERROR]"), msg.as_ref());
    }

    pub fn success<S: AsRef<str>>(msg: S) {
        println!("{} {}", Colors::success("[MONOX]"), msg.as_ref());
    }
}
