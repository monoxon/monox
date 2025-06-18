// ============================================================================
// MonoX - 国际化模块
// ============================================================================
//
// 文件: src/i18n/mod.rs
// 职责: 国际化支持和翻译管理
// 边界:
//   - ✅ 翻译表初始化和管理
//   - ✅ 翻译宏定义和实现
//   - ✅ 语言切换支持
//   - ✅ 参数化翻译支持
//   - ❌ 不应包含具体翻译内容
//   - ❌ 不应包含业务逻辑
//   - ❌ 不应包含 CLI 相关逻辑
//   - ❌ 不应包含文件操作逻辑
//
// ============================================================================

// 国际化模块

pub mod en_us;
pub mod zh_cn;

/// 获取翻译文本
pub fn get_translation(key: &str) -> String {
    // 每次都从配置获取语言设置
    let language = get_language_from_config().unwrap_or_else(|| "en_us".to_string());

    let translation_data = match language.as_str() {
        "zh_cn" => zh_cn::TRANSLATIONS,
        "en_us" | _ => en_us::TRANSLATIONS, // 默认使用英文
    };

    // 查找翻译
    for &(k, v) in translation_data {
        if k == key {
            return v.to_string();
        }
    }

    format!("Unknown translation key: {}", key)
}

/// 从配置获取语言设置
fn get_language_from_config() -> Option<String> {
    use crate::models::config::Config;

    // 尝试获取配置中的语言设置
    // 如果配置未初始化或获取失败，返回 None
    Config::get_language().ok()
}

/// 简单翻译宏
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::get_translation($key)
    };
}

/// 带参数翻译的辅助函数
pub fn format_with_args(template: String, args: Vec<String>) -> String {
    let mut result = template;
    for arg in args.iter() {
        // 替换第一个 {} 占位符
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, arg);
        }
    }
    result
}

/// 带参数的翻译宏
#[macro_export]
macro_rules! tf {
    ($key:expr, $($arg:expr),*) => {{
        let template = $crate::i18n::get_translation($key);
        let args = vec![$(format!("{}", $arg)),*];
        $crate::i18n::format_with_args(template, args)
    }};
}
