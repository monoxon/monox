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
use super::constants::APP_NAME;

/// 简单的日志工具
pub struct Logger;

impl Logger {
    pub fn debug<S: AsRef<str>>(msg: S) {
        println!(
            "{} {}",
            Colors::debug(&format!("[{}:DEBUG]", APP_NAME)),
            msg.as_ref()
        );
    }

    pub fn info<S: AsRef<str>>(msg: S) {
        println!(
            "{} {}",
            Colors::info(&format!("[{}]", APP_NAME)),
            msg.as_ref()
        );
    }

    pub fn warn<S: AsRef<str>>(msg: S) {
        println!(
            "{} {}",
            Colors::warn(&format!("[{}]", APP_NAME)),
            msg.as_ref()
        );
    }

    pub fn error<S: AsRef<str>>(msg: S) {
        eprintln!(
            "{} {}",
            Colors::error(&format!("[{}]", APP_NAME)),
            msg.as_ref()
        );
    }

    pub fn success<S: AsRef<str>>(msg: S) {
        println!(
            "{} {}",
            Colors::success(&format!("[{}]", APP_NAME)),
            msg.as_ref()
        );
    }
}
