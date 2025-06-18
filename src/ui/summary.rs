// ============================================================================
// MonoX - 执行结果汇总组件
// ============================================================================
//
// 文件: src/ui/summary.rs
// 职责: 执行结果汇总显示
// 边界:
//   - ✅ 执行结果汇总显示
//   - ✅ 统计信息格式化输出
//   - ✅ 国际化文本支持
//   - ❌ 不应包含具体业务逻辑
//   - ❌ 不应包含任务执行逻辑
//   - ❌ 不应包含文件操作
//   - ❌ 不应包含数据处理逻辑
//
// ============================================================================

use crate::utils::constants::icons;
use crate::utils::logger::Logger;
use crate::utils::styles::TextStyles;
use crate::{t, tf};
use std::io::{self, Write};

/// 渲染执行汇总
pub fn render_execution_summary(
    total_tasks: usize,
    successful_tasks: usize,
    failed_tasks: usize,
    skipped_tasks: usize,
    duration_ms: Option<u64>,
) {
    // 构建汇总内容
    let mut summary_lines = vec![
        "".to_string(),
        TextStyles::bold(&t!("runner.execution_summary")),
        "═══════════════════════════════════════".to_string(),
        format!(
            "{} {}",
            icons::PACKAGE,
            tf!("runner.total_tasks", total_tasks)
        ),
        format!(
            "{} {}",
            icons::SUCCESS,
            tf!("runner.successful_tasks", successful_tasks)
        ),
        format!(
            "{} {}",
            icons::ERROR,
            tf!("runner.failed_tasks", failed_tasks)
        ),
        format!(
            "{} {}",
            icons::SKIP,
            tf!("runner.skipped_tasks", skipped_tasks)
        ),
    ];

    // 如果有执行时长信息，添加到汇总中
    if let Some(duration) = duration_ms {
        summary_lines.push(format!(
            "{} {}",
            icons::TIME,
            tf!("executor.summary_duration", duration as f64 / 1000.0)
        ));
    }

    // 输出汇总内容
    for line in summary_lines {
        Logger::info(line);
    }

    let _ = io::stdout().flush();
}
