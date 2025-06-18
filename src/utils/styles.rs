// ============================================================================
// MonoX - 文本样式工具
// ============================================================================
//
// 文件: src/utils/styles.rs
// 职责: 终端文本样式格式化
// 边界:
//   - ✅ 文本样式代码定义（粗体、斜体、下划线等）
//   - ✅ 文本样式格式化
//   - ✅ 样式组合支持
//   - ❌ 不应包含颜色相关功能
//   - ❌ 不应包含业务逻辑
//   - ❌ 不应包含 UI 组件实现
//
// ============================================================================

/// ANSI 文本样式代码
pub mod ansi_styles {
    /// 重置所有样式
    pub const RESET: &str = "\x1b[0m";

    /// 粗体
    pub const BOLD: &str = "\x1b[1m";

    /// 斜体
    pub const ITALIC: &str = "\x1b[3m";

    /// 下划线
    pub const UNDERLINE: &str = "\x1b[4m";

    /// 删除线
    pub const STRIKETHROUGH: &str = "\x1b[9m";
}

/// 文本样式工具函数
pub struct TextStyles;

impl TextStyles {
    /// 为文本添加样式
    pub fn stylize(text: &str, style: &str) -> String {
        format!("{}{}{}", style, text, ansi_styles::RESET)
    }

    /// 粗体文本
    pub fn bold(text: &str) -> String {
        Self::stylize(text, ansi_styles::BOLD)
    }

    /// 斜体文本
    pub fn italic(text: &str) -> String {
        Self::stylize(text, ansi_styles::ITALIC)
    }

    /// 下划线文本
    pub fn underline(text: &str) -> String {
        Self::stylize(text, ansi_styles::UNDERLINE)
    }

    /// 删除线文本
    pub fn strikethrough(text: &str) -> String {
        Self::stylize(text, ansi_styles::STRIKETHROUGH)
    }
}
