// src/hotkey.rs
use log::{error, info};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use winapi::ctypes::c_int;
use winapi::shared::windef::HWND;
use winapi::um::winuser::{
    DispatchMessageW, GetMessageW, RegisterHotKey, TranslateMessage, UnregisterHotKey, MSG,
    WM_HOTKEY,
};

// 修饰键
pub const MOD_CONTROL: c_int = 0x0002;
pub const MOD_SHIFT: c_int = 0x0004;
pub const MOD_ALT: c_int = 0x0001;

type HotkeyCallback = Arc<Mutex<Box<dyn Fn() + Send>>>;

pub struct HotkeyManager {
    thread_handle: Option<thread::JoinHandle<()>>,
    registered: bool,
    exit_sender: Option<Sender<()>>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        HotkeyManager {
            thread_handle: None,
            registered: false,
            exit_sender: None,
        }
    }

    pub fn register<F>(&mut self, modifiers: c_int, key: c_int, callback: F) -> Result<(), String>
    where
        F: Fn() + Send + 'static,
    {
        if self.registered {
            return Err("热键已注册".into());
        }

        // 使用Arc<Mutex<>>包装回调函数，使其可以跨线程安全共享
        let callback = Arc::new(Mutex::new(Box::new(callback) as Box<dyn Fn() + Send>));

        // 创建一个通道用于通知线程退出
        let (exit_tx, exit_rx) = channel();
        self.exit_sender = Some(exit_tx);

        // 线程安全地克隆回调，以便在线程内使用
        let callback_clone = Arc::clone(&callback);

        let thread_handle = thread::spawn(move || {
            unsafe {
                // 注册热键 (ID: 1)
                if RegisterHotKey(std::ptr::null_mut(), 1, modifiers as u32, key as u32) == 0 {
                    error!("无法注册热键");
                    return;
                }

                info!("热键注册成功");

                // 消息循环
                let mut msg: MSG = std::mem::zeroed();
                let mut should_exit = false;

                while !should_exit && GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
                    // 检查是否收到退出信号
                    if let Ok(_) = exit_rx.try_recv() {
                        should_exit = true;
                        break;
                    }

                    if msg.message == WM_HOTKEY && msg.wParam == 1 {
                        info!("热键触发");

                        // 执行回调
                        if let Ok(callback) = callback_clone.lock() {
                            (*callback)();
                        }
                    }

                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // 注销热键
                UnregisterHotKey(std::ptr::null_mut(), 1);
            }
        });

        self.thread_handle = Some(thread_handle);
        self.registered = true;
        Ok(())
    }

    pub fn unregister(&mut self) -> Result<(), String> {
        if !self.registered {
            return Ok(());
        }

        // 发送退出信号
        if let Some(sender) = &self.exit_sender {
            if let Err(_) = sender.send(()) {
                error!("无法发送退出信号");
            }
        }

        self.registered = false;
        self.exit_sender = None;

        info!("热键已注销");
        Ok(())
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        if self.registered {
            let _ = self.unregister();
        }
    }
}
