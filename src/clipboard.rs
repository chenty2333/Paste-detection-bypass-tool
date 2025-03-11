// src/clipboard.rs
use log::{error, info};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr::null_mut;
use winapi::shared::minwindef::HGLOBAL;
use winapi::um::winbase::{GlobalLock, GlobalUnlock};
use winapi::um::winuser::{CloseClipboard, GetClipboardData, OpenClipboard, CF_UNICODETEXT};

pub struct ClipboardManager {
    buffer: Option<String>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        ClipboardManager { buffer: None }
    }

    pub fn capture_clipboard(&mut self) -> Result<(), String> {
        unsafe {
            // 打开剪贴板
            if OpenClipboard(null_mut()) == 0 {
                error!("无法打开剪贴板");
                return Err("无法打开剪贴板".into());
            }

            // 获取剪贴板数据
            let h_data = GetClipboardData(CF_UNICODETEXT);
            if h_data.is_null() {
                CloseClipboard();
                error!("剪贴板中没有文本数据");
                return Err("剪贴板中没有文本数据".into());
            }

            // 锁定内存并获取指向文本的指针
            let p_data = GlobalLock(h_data as HGLOBAL);
            if p_data.is_null() {
                CloseClipboard();
                error!("无法锁定剪贴板内存");
                return Err("无法锁定剪贴板内存".into());
            }

            // 转换为Rust字符串
            let wide_text = std::slice::from_raw_parts(
                p_data as *const u16,
                (0..)
                    .take_while(|&i| *((p_data as *const u16).offset(i)) != 0)
                    .count(),
            );

            let result = OsString::from_wide(wide_text)
                .to_string_lossy()
                .into_owned();

            // 更新缓冲区
            self.buffer = Some(result);

            // 解锁并关闭
            GlobalUnlock(h_data as HGLOBAL);
            CloseClipboard();

            info!(
                "已捕获剪贴板内容，长度: {}",
                self.buffer.as_ref().unwrap().len()
            );
            Ok(())
        }
    }

    pub fn get_buffer(&self) -> Option<&String> {
        self.buffer.as_ref()
    }

    pub fn clear_buffer(&mut self) {
        self.buffer = None;
        info!("缓冲区已清空");
    }
}
