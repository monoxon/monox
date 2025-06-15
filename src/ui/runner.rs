// ============================================================================
// MonoX - 任务执行 UI 组件
// ============================================================================
//
// 文件: src/ui/runner.rs
// 职责: 任务运行器的用户界面显示
// 边界:
//   - ✅ 任务执行进度显示
//   - ✅ 实时输出流显示
//   - ✅ 任务状态指示器
//   - ✅ 执行统计信息显示
//   - ✅ 错误和警告高亮
//   - ✅ 多任务并行显示
//   - ❌ 不应包含任务执行逻辑
//   - ❌ 不应包含业务数据处理
//   - ❌ 不应包含文件操作逻辑
//   - ❌ 不应包含配置管理逻辑
//
// ============================================================================

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 任务执行状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
    /// 已跳过
    Skipped,
}

/// 任务执行信息
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// 任务名称
    pub name: String,
    /// 包名
    pub package: String,
    /// 执行状态
    pub status: TaskStatus,
    /// 开始时间
    pub start_time: Option<Instant>,
    /// 结束时间
    pub end_time: Option<Instant>,
    /// 输出日志
    pub output: Vec<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 任务运行器 UI 主组件
pub struct RunnerUI {
    /// 所有任务信息
    tasks: HashMap<String, TaskInfo>,
    /// 当前执行阶段
    current_stage: usize,
    /// 总阶段数
    total_stages: usize,
    /// 是否显示详细输出
    verbose: bool,
    /// 是否启用彩色输出
    colored: bool,
    /// 是否显示进度条
    show_progress: bool,
}

impl RunnerUI {
    /// 创建新的任务 UI
    pub fn new(verbose: bool, colored: bool, show_progress: bool) -> Self {
        Self {
            tasks: HashMap::new(),
            current_stage: 0,
            total_stages: 0,
            verbose,
            colored,
            show_progress,
        }
    }

    /// 设置总阶段数
    pub fn set_total_stages(&mut self, total: usize) {
        self.total_stages = total;
    }

    /// 开始新阶段
    pub fn start_stage(&mut self, stage: usize) {
        self.current_stage = stage;
        if self.show_progress {
            self.render_stage_header();
        }
    }

    /// 添加任务
    pub fn add_task(&mut self, task_id: String, name: String, package: String) {
        let task_info = TaskInfo {
            name,
            package,
            status: TaskStatus::Pending,
            start_time: None,
            end_time: None,
            output: Vec::new(),
            error: None,
        };
        self.tasks.insert(task_id, task_info);
    }

    /// 开始执行任务
    pub fn start_task(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Running;
            task.start_time = Some(Instant::now());

            // 克隆任务信息以避免借用冲突
            let task_clone = task.clone();
            self.render_task_start(&task_clone);
        }
    }

    /// 任务执行成功
    pub fn complete_task(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Success;
            task.end_time = Some(Instant::now());

            // 克隆任务信息以避免借用冲突
            let task_clone = task.clone();
            self.render_task_complete(&task_clone);
        }
    }

    /// 任务执行失败
    pub fn fail_task(&mut self, task_id: &str, error: String) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Failed;
            task.end_time = Some(Instant::now());
            task.error = Some(error);

            // 克隆任务信息以避免借用冲突
            let task_clone = task.clone();
            self.render_task_failed(&task_clone);
        }
    }

    /// 添加任务输出
    pub fn add_task_output(&mut self, task_id: &str, output: String) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.output.push(output.clone());
            if self.verbose {
                // 克隆任务信息以避免借用冲突
                let task_clone = task.clone();
                self.render_task_output(&task_clone, &output);
            }
        }
    }

    /// 渲染阶段头部
    fn render_stage_header(&self) {
        use crate::utils::constants::icons;
        use crate::utils::logger::Logger;
        use crate::{t, tf};

        Logger::info(format!(
            "\n{} {}",
            icons::STAGE,
            tf!("runner.stage_header", self.current_stage, self.total_stages)
        ));
    }

    /// 渲染任务开始
    fn render_task_start(&self, task: &TaskInfo) {
        if !self.verbose {
            return;
        }

        use crate::utils::constants::icons;
        use crate::utils::logger::Logger;
        use crate::{t, tf};

        Logger::info(format!(
            "  {} {}",
            icons::EXEC,
            tf!("runner.task_start", task.name, task.package)
        ));
    }

    /// 渲染任务完成
    fn render_task_complete(&self, task: &TaskInfo) {
        use crate::utils::constants::icons;
        use crate::utils::logger::Logger;
        use crate::{t, tf};

        let duration = if let (Some(start), Some(end)) = (task.start_time, task.end_time) {
            end.duration_since(start)
        } else {
            Duration::from_secs(0)
        };

        Logger::info(format!(
            "  {} {}",
            icons::SUCCESS,
            tf!(
                "runner.task_complete",
                task.name,
                task.package,
                duration.as_millis()
            )
        ));
    }

    /// 渲染任务失败
    fn render_task_failed(&self, task: &TaskInfo) {
        use crate::utils::constants::icons;
        use crate::utils::logger::Logger;
        use crate::{t, tf};

        Logger::error(format!(
            "  {} {}",
            icons::ERROR,
            tf!("runner.task_failed", task.name, task.package)
        ));

        if let Some(error) = &task.error {
            Logger::error(format!("    {}", error));
        }
    }

    /// 渲染任务输出
    fn render_task_output(&self, task: &TaskInfo, output: &str) {
        use crate::utils::logger::Logger;

        Logger::info(format!("    [{}] {}", task.package, output));
    }

    /// 渲染执行总结
    pub fn render_summary(&self) {
        use crate::utils::constants::icons;
        use crate::utils::logger::Logger;
        use crate::{t, tf};

        let total_tasks = self.tasks.len();
        let successful_tasks = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Success)
            .count();
        let failed_tasks = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Failed)
            .count();

        Logger::info(format!(
            "\n{} {}",
            icons::COMPLETE,
            t!("runner.execution_summary")
        ));
        Logger::info("═══════════════════════════════════════");
        Logger::info(format!(
            "{} {}",
            icons::PACKAGE,
            tf!("runner.total_tasks", total_tasks)
        ));
        Logger::info(format!(
            "{} {}",
            icons::SUCCESS,
            tf!("runner.successful_tasks", successful_tasks)
        ));

        if failed_tasks > 0 {
            Logger::error(format!(
                "{} {}",
                icons::ERROR,
                tf!("runner.failed_tasks", failed_tasks)
            ));
        }
    }
}

/// 进度条组件（TaskUI 的子组件）
pub struct ProgressBar {
    /// 当前进度
    current: usize,
    /// 总数
    total: usize,
    /// 进度条宽度
    width: usize,
    /// 是否启用彩色
    colored: bool,
}

impl ProgressBar {
    /// 创建新的进度条
    pub fn new(total: usize, width: usize, colored: bool) -> Self {
        Self {
            current: 0,
            total,
            width,
            colored,
        }
    }

    /// 更新进度
    pub fn update(&mut self, current: usize) {
        self.current = current;
    }

    /// 渲染进度条
    pub fn render(&self) -> String {
        use crate::utils::constants::progress_chars;

        if self.total == 0 {
            return String::new();
        }

        let percentage = (self.current as f64 / self.total as f64 * 100.0) as usize;
        let filled_width = (self.current as f64 / self.total as f64 * self.width as f64) as usize;
        let empty_width = self.width - filled_width;

        let filled_part = progress_chars::FILLED.repeat(filled_width);
        let empty_part = progress_chars::EMPTY.repeat(empty_width);

        format!(
            "[{}{}] {}/{} ({}%)",
            filled_part, empty_part, self.current, self.total, percentage
        )
    }
}
