// ============================================================================
// MonoX - 常量定义
// ============================================================================
//
// 文件: src/utils/constants.rs
// 职责: 应用程序常量和配置定义
// 边界:
//   - ✅ 应用程序常量定义
//   - ✅ 像素图标字符定义
//   - ✅ 颜色主题定义
//   - ✅ UI 相关常量定义
//   - ❌ 不应包含动态配置
//   - ❌ 不应包含业务逻辑
//   - ❌ 不应包含计算逻辑
//   - ❌ 不应包含文件路径处理
//
// ============================================================================

/// 应用名称常量
pub const APP_NAME: &str = "MONOX";

/// 像素风格图标
pub mod icons {
    /// 构建图标
    pub const BUILD: &str = "▓";
    /// 成功图标
    pub const SUCCESS: &str = "✓";
    /// 错误图标
    pub const ERROR: &str = "✗";
    /// 警告图标
    pub const WARNING: &str = "!";
    /// 信息图标
    pub const INFO: &str = "i";
    /// 包图标
    pub const PACKAGE: &str = "●";
    /// 阶段图标
    pub const STAGE: &str = "▪";
    /// 完成图标
    pub const COMPLETE: &str = "●";
    /// 检查图标
    pub const CHECK: &str = "◆";
    /// 分析图标
    pub const ANALYZE: &str = "◇";
    /// 更新图标
    pub const UPDATE: &str = "▲";
    /// 初始化图标
    pub const INIT: &str = "◈";
    /// 执行图标
    pub const EXEC: &str = "▸";
    /// 依赖图标
    pub const DEPENDENCY: &str = "◦";
    /// 目标图标
    pub const TARGET: &str = "◉";
    /// 时间图标
    pub const TIME: &str = "⧖";
    /// 性能图标
    pub const PERF: &str = "⧗";
    /// 箭头图标
    pub const ARROW: &str = "→";
    /// 汇总图标
    pub const SUMMARY: &str = "◈";
    /// 跳过图标
    pub const SKIP: &str = "○";
}

/// 进度条字符
pub mod progress_chars {
    /// 已完成块
    pub const FILLED: &str = "█";
    /// 未完成块
    pub const EMPTY: &str = "░";
}

/// 加载 spinner 字符
pub mod spinner_chars {
    pub const BASE: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];
}

/// 颜色主题
pub mod colors {
    /// 主色调 - 青色
    pub const PRIMARY: &str = "cyan";
    /// 成功色 - 绿色
    pub const SUCCESS: &str = "green";
    /// 错误色 - 红色
    pub const ERROR: &str = "red";
    /// 警告色 - 黄色
    pub const WARNING: &str = "yellow";
    /// 信息色 - 蓝色
    pub const INFO: &str = "blue";
    /// 次要色 - 灰色
    pub const SECONDARY: &str = "bright_black";
    /// 高亮色 - 白色
    pub const HIGHLIGHT: &str = "white";
    /// 进度条颜色 - 亮青色
    pub const PROGRESS: &str = "bright_cyan";
}
