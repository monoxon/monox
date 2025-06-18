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

use crate::utils::constants::{icons, progress_chars, spinner_chars};
use crate::utils::logger::Logger;
use crate::{t, tf};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    /// 已渲染的行数（用于清除屏幕）
    rendered_lines: usize,
    /// 是否支持终端刷新
    supports_refresh: bool,
    /// Spinner 动画帧
    spinner_frame: usize,
    /// 当前阶段的包列表
    current_stage_packages: Vec<String>,
    /// 自动刷新定时器控制
    refresh_timer_running: Arc<AtomicBool>,
    /// 定时器线程句柄
    refresh_timer_handle: Option<thread::JoinHandle<()>>,
    /// 自引用（用于定时器回调）
    self_ref: Option<Weak<Mutex<RunnerUI>>>,
}

impl RunnerUI {
    /// 创建新的任务 UI
    pub fn new(verbose: bool, colored: bool, show_progress: bool) -> Self {
        let supports_refresh = !verbose && atty::is(atty::Stream::Stdout);

        Self {
            tasks: HashMap::new(),
            current_stage: 0,
            total_stages: 0,
            verbose,
            colored,
            show_progress,
            rendered_lines: 0,
            supports_refresh,
            spinner_frame: 0,
            current_stage_packages: Vec::new(),
            refresh_timer_running: Arc::new(AtomicBool::new(false)),
            refresh_timer_handle: None,
            self_ref: None,
        }
    }

    /// 设置自引用（在创建 Arc<Mutex<RunnerUI>> 后调用）
    pub fn set_self_ref(&mut self, self_ref: Weak<Mutex<RunnerUI>>) {
        self.self_ref = Some(self_ref);
    }

    /// 设置总阶段数
    pub fn set_total_stages(&mut self, total: usize) {
        self.total_stages = total;
    }

    /// 开始新阶段
    pub fn start_stage(&mut self, stage: usize) {
        self.current_stage = stage;

        // 收集当前阶段的包列表
        self.current_stage_packages = self
            .tasks
            .values()
            .filter(|task| {
                // 这里需要根据实际情况判断哪些任务属于当前阶段
                // 暂时显示所有任务的包名
                true
            })
            .map(|task| task.package.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        self.current_stage_packages.sort();

        if self.show_progress && !self.verbose {
            self.refresh_display();
        } else if self.verbose {
            self.render_stage_header_verbose();
        }
    }

    /// 设置当前阶段的包列表
    pub fn set_stage_packages(&mut self, packages: Vec<String>) {
        self.current_stage_packages = packages;
        if self.supports_refresh && !self.verbose {
            self.refresh_display();
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

            if self.verbose {
                let task_clone = task.clone();
                self.render_task_start(&task_clone);
            } else {
                self.start_refresh_timer();
                self.refresh_display();
            }
        }
    }

    /// 任务执行成功
    pub fn complete_task(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Success;
            task.end_time = Some(Instant::now());

            if self.verbose {
                let task_clone = task.clone();
                self.render_task_complete(&task_clone);
            } else {
                self.refresh_display();
                // 检查是否所有任务都完成了
                if !self.has_running_tasks() {
                    self.stop_refresh_timer();
                }
            }
        }
    }

    /// 任务执行失败
    pub fn fail_task(&mut self, task_id: &str, error: String) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = TaskStatus::Failed;
            task.end_time = Some(Instant::now());
            task.error = Some(error);

            if self.verbose {
                let task_clone = task.clone();
                self.render_task_failed(&task_clone);
            } else {
                self.refresh_display();
                // 检查是否所有任务都完成了
                if !self.has_running_tasks() {
                    self.stop_refresh_timer();
                }
            }
        }
    }

    /// 刷新整个显示（非 verbose 模式）
    fn refresh_display(&mut self) {
        if !self.supports_refresh {
            return;
        }

        // 清除之前的输出
        self.clear_screen();

        // 自动更新 spinner 帧（基于时间）
        let now = SystemTime::now();
        let elapsed_ms = now
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis();
        self.spinner_frame = ((elapsed_ms / 100) % 8) as usize;

        // 重新渲染
        let content = self.build_display_content();
        print!("{}", content);
        let _ = io::stdout().flush();

        // 记录渲染的行数
        self.rendered_lines = content.lines().count();
    }

    /// 清除屏幕
    fn clear_screen(&self) {
        if self.rendered_lines > 0 {
            // 移动光标到之前渲染内容的开始位置
            print!("\x1B[{}A", self.rendered_lines);
            // 清除从光标到屏幕底部的内容
            print!("\x1B[J");
        }
    }

    /// 构建显示内容
    fn build_display_content(&self) -> String {
        let mut content = String::new();

        if self.current_stage > 0 && self.total_stages > 0 {
            // Spinner 动画
            let spinner_char = spinner_chars::BASE[self.spinner_frame];

            // 进度条
            let progress_bar = self.build_progress_bar();

            // 阶段头部：Spinner + 进度条 + Stage 信息
            content.push_str(&format!(
                "{} {} {}\n",
                spinner_char,
                progress_bar,
                tf!("runner.stage_header", self.current_stage, self.total_stages)
            ));

            // 当前阶段包列表
            if !self.current_stage_packages.is_empty() {
                content.push_str(&format!("{}\n", t!("runner.processing_packages")));
                for (i, package) in self.current_stage_packages.iter().enumerate() {
                    let status_icon = self.get_package_status_icon(package);
                    content.push_str(&format!("  {} {}\n", status_icon, package));

                    // 限制显示数量，避免屏幕过满
                    if i >= 10 {
                        let remaining = self.current_stage_packages.len() - i - 1;
                        if remaining > 0 {
                            content
                                .push_str(&format!("{}\n", tf!("runner.more_packages", remaining)));
                        }
                        break;
                    }
                }
            }
        }

        content
    }

    /// 构建进度条
    fn build_progress_bar(&self) -> String {
        if self.total_stages == 0 {
            return String::new();
        }

        let width = 20; // 进度条宽度
        let progress = (self.current_stage as f64 / self.total_stages as f64).min(1.0);
        let filled_width = (progress * width as f64) as usize;
        let empty_width = width - filled_width;

        let filled_part = progress_chars::FILLED.repeat(filled_width);
        let empty_part = progress_chars::EMPTY.repeat(empty_width);

        format!("[{}{}]", filled_part, empty_part)
    }

    /// 获取包的状态图标
    fn get_package_status_icon(&self, package: &str) -> &'static str {
        use crate::utils::constants::icons;

        // 查找该包的任务状态
        for task in self.tasks.values() {
            if task.package == package {
                return match task.status {
                    TaskStatus::Running => "▸",
                    TaskStatus::Success => icons::SUCCESS,
                    TaskStatus::Failed => icons::ERROR,
                    TaskStatus::Pending => "○",
                    TaskStatus::Skipped => icons::SKIP,
                };
            }
        }

        "○" // 默认待处理状态
    }

    /// 检查是否有正在运行的任务
    fn has_running_tasks(&self) -> bool {
        self.tasks
            .values()
            .any(|task| task.status == TaskStatus::Running)
    }

    /// 启动自动刷新定时器
    fn start_refresh_timer(&mut self) {
        if !self.supports_refresh || self.refresh_timer_running.load(Ordering::Relaxed) {
            return;
        }

        self.refresh_timer_running.store(true, Ordering::Relaxed);

        // 获取自引用的弱指针
        if let Some(self_weak) = self.self_ref.clone() {
            let timer_running = Arc::clone(&self.refresh_timer_running);

            let handle = thread::spawn(move || {
                while timer_running.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));

                    // 尝试升级弱引用并刷新显示
                    if let Some(ui_arc) = self_weak.upgrade() {
                        if let Ok(mut ui) = ui_arc.try_lock() {
                            // 检查是否还有运行中的任务
                            if ui.has_running_tasks() && ui.supports_refresh {
                                ui.refresh_display();
                            } else if !ui.has_running_tasks() {
                                // 如果没有运行中的任务，停止定时器
                                timer_running.store(false, Ordering::Relaxed);
                                break;
                            }
                        }
                    } else {
                        // UI 已被销毁，停止定时器
                        break;
                    }
                }
            });

            self.refresh_timer_handle = Some(handle);
        }
    }

    /// 停止自动刷新定时器
    fn stop_refresh_timer(&mut self) {
        self.refresh_timer_running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.refresh_timer_handle.take() {
            let _ = handle.join();
        }
    }

    /// 渲染阶段头部（verbose 模式）
    fn render_stage_header_verbose(&self) {
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

        Logger::info(format!(
            "  {} {}",
            icons::EXEC,
            tf!("runner.task_start", task.name, task.package)
        ));
    }

    /// 渲染任务完成
    fn render_task_complete(&self, task: &TaskInfo) {
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

    /// 渲染执行总结
    pub fn render_summary(&mut self) {
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
        let skipped_tasks = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Skipped)
            .count();

        if self.supports_refresh && !self.verbose {
            // 清除刷新模式的显示
            self.clear_screen();

            // 显示最终汇总
            let mut summary_lines = vec![
                format!("\n{} {}", icons::COMPLETE, t!("runner.execution_summary")),
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
                format!("○ {}", tf!("runner.skipped_tasks", skipped_tasks)),
            ];

            let summary_content = summary_lines.join("\n") + "\n";

            print!("{}", summary_content);
            let _ = io::stdout().flush();
            self.rendered_lines = 0; // 重置，不再清除这个输出
        } else {
            // Verbose 模式使用 Logger
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

            if skipped_tasks > 0 {
                Logger::warn(format!("○ {}", tf!("runner.skipped_tasks", skipped_tasks)));
            }
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

impl Drop for RunnerUI {
    fn drop(&mut self) {
        self.stop_refresh_timer();
    }
}
