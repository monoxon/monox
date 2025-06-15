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
    /// 当前消息
    message: Arc<Mutex<String>>,
    /// 线程句柄
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    /// 创建新的 Spinner
    pub fn new(message: String) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            message: Arc::new(Mutex::new(message)),
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
        let message = Arc::clone(&self.message);

        let handle = thread::spawn(move || {
            let mut frame = 0;

            while running.load(Ordering::Relaxed) {
                let spinner_char = spinner_chars::BASE[frame % spinner_chars::BASE.len()];
                let msg = message.lock().unwrap().clone();

                // 清除当前行并打印新内容
                print!("\r{} {}", spinner_char, msg);
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

    /// 更新消息
    pub fn update_message(&self, new_message: String) {
        if let Ok(mut msg) = self.message.lock() {
            *msg = new_message;
        }
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
