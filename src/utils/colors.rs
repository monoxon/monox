// ============================================================================
// MonoX - 颜色工具
// ============================================================================
//
// 文件: src/utils/colors.rs
// 职责: 终端颜色输出和主题管理
// 边界:
//   - ✅ 终端颜色代码定义
//   - ✅ 颜色输出格式化
//   - ✅ 主题颜色管理
//   - ✅ 颜色兼容性处理
//   - ❌ 不应包含业务逻辑
//   - ❌ 不应包含 UI 组件实现
//   - ❌ 不应包含文本内容处理
//   - ❌ 不应包含特定领域逻辑
//
// ============================================================================

/// ANSI 颜色代码
pub mod ansi {
    /// 重置颜色
    pub const RESET: &str = "\x1b[0m";

    /// 前景色
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
}

/// 日志级别颜色主题
pub mod log_colors {
    use super::ansi;

    /// 信息日志颜色 (青色)
    pub const INFO: &str = ansi::CYAN;

    /// 警告日志颜色 (黄色)
    pub const WARN: &str = ansi::YELLOW;

    /// 错误日志颜色 (红色)
    pub const ERROR: &str = ansi::RED;

    /// 成功日志颜色 (绿色)
    pub const SUCCESS: &str = ansi::GREEN;
}

/// 颜色工具函数
pub struct Colors;

impl Colors {
    /// 为文本添加颜色
    pub fn colorize(text: &str, color: &str) -> String {
        format!("{}{}{}", color, text, ansi::RESET)
    }

    /// 信息颜色
    pub fn info(text: &str) -> String {
        Self::colorize(text, log_colors::INFO)
    }

    /// 警告颜色
    pub fn warn(text: &str) -> String {
        Self::colorize(text, log_colors::WARN)
    }

    /// 错误颜色
    pub fn error(text: &str) -> String {
        Self::colorize(text, log_colors::ERROR)
    }

    /// 成功颜色
    pub fn success(text: &str) -> String {
        Self::colorize(text, log_colors::SUCCESS)
    }
}
