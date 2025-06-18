// ============================================================================
// MonoX - Spinner 加载动画组件
// ============================================================================
//
// 文件: src/utils/spinner.rs
// 职责: 终端加载动画显示组件
// 边界:
//   - ✅ 加载动画显示和控制
//   - ✅ 多线程安全的状态管理
//   - ✅ 自定义消息和进度更新
//   - ✅ 优雅的启动和停止机制
//   - ❌ 不应包含业务逻辑
//   - ❌ 不应包含文件操作
//   - ❌ 不应包含网络请求
//   - ❌ 不应包含数据处理逻辑
//
// ============================================================================

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::utils::constants::spinner_chars;

/// Spinner 加载动画组件
pub struct Spinner {
    /// 是否正在运行
    running: Arc<AtomicBool>,
    /// 前缀消息
    prefix: Arc<Mutex<String>>,
    /// 后缀消息  
    suffix: Arc<Mutex<String>>,
    /// 线程句柄
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// 创建新的 Spinner（只有后缀消息，保持向后兼容）
    pub fn new(message: String) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            prefix: Arc::new(Mutex::new(String::new())),
            suffix: Arc::new(Mutex::new(message)),
            handle: None,
        }
    }

    /// 创建带前缀和后缀的 Spinner
    pub fn new_with_prefix(prefix: String, suffix: String) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            prefix: Arc::new(Mutex::new(prefix)),
            suffix: Arc::new(Mutex::new(suffix)),
            handle: None,
        }
    }

    /// 启动 Spinner
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(true, Ordering::Relaxed);

        let running = Arc::clone(&self.running);
        let prefix = Arc::clone(&self.prefix);
        let suffix = Arc::clone(&self.suffix);

        let handle = thread::spawn(move || {
            let mut frame = 0;

            while running.load(Ordering::Relaxed) {
                let spinner_char = spinner_chars::BASE[frame % spinner_chars::BASE.len()];
                let prefix_msg = prefix.lock().unwrap().clone();
                let suffix_msg = suffix.lock().unwrap().clone();

                // 构建显示文本：prefix + spinner + suffix
                let display_text = if prefix_msg.is_empty() {
                    if suffix_msg.is_empty() {
                        format!("{}", spinner_char)
                    } else {
                        format!("{} {}", spinner_char, suffix_msg)
                    }
                } else {
                    if suffix_msg.is_empty() {
                        format!("{} {}", prefix_msg, spinner_char)
                    } else {
                        format!("{} {} {}", prefix_msg, spinner_char, suffix_msg)
                    }
                };

                // 清除当前行并打印新内容
                print!("\r{}", display_text);
                io::stdout().flush().unwrap();

                frame += 1;
                thread::sleep(Duration::from_millis(100));
            }

            // 清除 spinner 行
            print!("\r");
            io::stdout().flush().unwrap();
        });

        self.handle = Some(handle);
    }

    /// 更新后缀消息（保持向后兼容）
    pub fn update_message(&self, new_message: String) {
        if let Ok(mut msg) = self.suffix.lock() {
            *msg = new_message;
        }
    }

    /// 更新前缀消息
    pub fn update_prefix(&self, new_prefix: String) {
        if let Ok(mut prefix) = self.prefix.lock() {
            *prefix = new_prefix;
        }
    }

    /// 更新后缀消息
    pub fn update_suffix(&self, new_suffix: String) {
        if let Ok(mut suffix) = self.suffix.lock() {
            *suffix = new_suffix;
        }
    }

    /// 同时更新前缀和后缀消息
    pub fn update_both(&self, new_prefix: String, new_suffix: String) {
        if let Ok(mut prefix) = self.prefix.lock() {
            *prefix = new_prefix;
        }
        if let Ok(mut suffix) = self.suffix.lock() {
            *suffix = new_suffix;
        }
    }

    /// 手动更新显示（用于外部控制的场景）
    /// 该方法会立即更新显示而不依赖内部循环
    pub fn manual_update(&self, new_message: String, frame: usize) {
        if let Ok(mut msg) = self.suffix.lock() {
            *msg = new_message.clone();
        }

        let spinner_char = spinner_chars::BASE[frame % spinner_chars::BASE.len()];
        let prefix_msg = self.prefix.lock().unwrap().clone();

        // 构建显示文本：prefix + spinner + suffix
        let display_text = if prefix_msg.is_empty() {
            format!("{} {}", spinner_char, new_message)
        } else {
            if new_message.is_empty() {
                format!("{} {}", prefix_msg, spinner_char)
            } else {
                format!("{} {} {}", prefix_msg, spinner_char, new_message)
            }
        };

        print!("\r{}", display_text);
        io::stdout().flush().unwrap();
    }

    /// 手动更新显示（支持前缀和后缀）
    pub fn manual_update_with_prefix(&self, new_prefix: String, new_suffix: String, frame: usize) {
        if let Ok(mut prefix) = self.prefix.lock() {
            *prefix = new_prefix.clone();
        }
        if let Ok(mut suffix) = self.suffix.lock() {
            *suffix = new_suffix.clone();
        }

        let spinner_char = spinner_chars::BASE[frame % spinner_chars::BASE.len()];

        // 构建显示文本：prefix + spinner + suffix
        let display_text = if new_prefix.is_empty() {
            if new_suffix.is_empty() {
                format!("{}", spinner_char)
            } else {
                format!("{} {}", spinner_char, new_suffix)
            }
        } else {
            if new_suffix.is_empty() {
                format!("{} {}", new_prefix, spinner_char)
            } else {
                format!("{} {} {}", new_prefix, spinner_char, new_suffix)
            }
        };

        print!("\r{}", display_text);
        io::stdout().flush().unwrap();
    }

    /// 清除当前显示行
    pub fn clear_line(&self) {
        print!("\r");
        io::stdout().flush().unwrap();
    }

    /// 停止 Spinner
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return;
        }

        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    /// 停止并显示最终消息
    pub fn finish_with_message(&mut self, final_message: String) {
        self.stop();
        println!("{}", final_message);
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}
