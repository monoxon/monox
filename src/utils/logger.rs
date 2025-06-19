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
    /// 获取各种类型日志前缀(带颜色)
    pub fn get_prefix(level: &str) -> String {
        match level {
            "DEBUG" => Colors::debug(&format!("[{}:DEBUG]", APP_NAME)),
            "INFO" => Colors::info(&format!("[{}]", APP_NAME)),
            "WARN" => Colors::warn(&format!("[{}]", APP_NAME)),
            "ERROR" => Colors::error(&format!("[{}]", APP_NAME)),
            "SUCCESS" => Colors::success(&format!("[{}]", APP_NAME)),
            _ => format!("[{}]", APP_NAME),
        }
    }

    pub fn debug<S: AsRef<str>>(msg: S) {
        println!("{} {}", Self::get_prefix("DEBUG"), msg.as_ref());
    }

    pub fn info<S: AsRef<str>>(msg: S) {
        println!("{} {}", Self::get_prefix("INFO"), msg.as_ref());
    }

    pub fn warn<S: AsRef<str>>(msg: S) {
        println!("{} {}", Self::get_prefix("WARN"), msg.as_ref());
    }

    pub fn error<S: AsRef<str>>(msg: S) {
        eprintln!("{} {}", Self::get_prefix("ERROR"), msg.as_ref());
    }

    pub fn success<S: AsRef<str>>(msg: S) {
        println!("{} {}", Self::get_prefix("SUCCESS"), msg.as_ref());
    }
}
