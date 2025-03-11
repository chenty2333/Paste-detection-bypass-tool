// src/input_sim.rs
use log::{error, info};
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{UINT, WORD};
use winapi::um::winuser::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_RETURN,
};

// 定义输入速度模式
#[derive(Debug, Clone, Copy)]
pub enum InputSpeed {
    Slow,   // 慢速 - 较长延迟，用于需要非常小心的网站
    Normal, // 正常 - 平衡的速度和安全性
    Fast,   // 快速 - 较短延迟，用于宽松的网站
    Turbo,  // 极速 - 批处理大量字符，几乎无延迟
}

pub struct InputSimulator {
    delay_ms: u64,          // 字符间延迟（毫秒）
    batch_size: usize,      // 批处理字符数量
    batch_delay_ms: u64,    // 批次间延迟（毫秒）
    speed_mode: InputSpeed, // 速度模式
}

impl InputSimulator {
    pub fn new() -> Self {
        InputSimulator {
            delay_ms: 10,      // 默认字符延迟10毫秒
            batch_size: 1,     // 默认单字符处理
            batch_delay_ms: 0, // 默认无批次延迟
            speed_mode: InputSpeed::Normal,
        }
    }

    pub fn with_delay(delay_ms: u64) -> Self {
        InputSimulator {
            delay_ms,
            batch_size: 1,
            batch_delay_ms: 0,
            speed_mode: InputSpeed::Normal,
        }
    }

    // 设置速度模式
    pub fn set_speed_mode(&mut self, mode: InputSpeed) {
        self.speed_mode = mode;
        match mode {
            InputSpeed::Slow => {
                self.delay_ms = 20;
                self.batch_size = 1;
                self.batch_delay_ms = 300;
            }
            InputSpeed::Normal => {
                self.delay_ms = 10;
                self.batch_size = 5;
                self.batch_delay_ms = 50;
            }
            InputSpeed::Fast => {
                self.delay_ms = 2;
                self.batch_size = 20;
                self.batch_delay_ms = 20;
            }
            InputSpeed::Turbo => {
                self.delay_ms = 0;
                self.batch_size = 50;
                self.batch_delay_ms = 10;
            }
        }

        info!(
            "速度模式设置为: {:?}, 延迟: {}ms, 批处理: {}, 批次延迟: {}ms",
            self.speed_mode, self.delay_ms, self.batch_size, self.batch_delay_ms
        );
    }

    // 设置自定义参数
    pub fn set_custom_params(&mut self, delay_ms: u64, batch_size: usize, batch_delay_ms: u64) {
        self.delay_ms = delay_ms;
        self.batch_size = batch_size;
        self.batch_delay_ms = batch_delay_ms;
        info!(
            "设置自定义参数: 延迟: {}ms, 批处理: {}, 批次延迟: {}ms",
            delay_ms, batch_size, batch_delay_ms
        );
    }

    pub fn set_delay(&mut self, delay_ms: u64) {
        self.delay_ms = delay_ms;
        info!("按键延迟设置为 {} 毫秒", delay_ms);
    }

    // 优化版的模拟输入函数
    pub fn simulate_typing(&self, text: &str) -> Result<(), String> {
        if text.is_empty() {
            return Ok(());
        }

        info!("开始模拟输入 {} 个字符", text.len());
        let start_time = std::time::Instant::now();

        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        let mut chars_processed = 0;

        // 分批处理字符
        for chunk in chars.chunks(self.batch_size) {
            let mut inputs = Vec::with_capacity(chunk.len() * 2); // 每个字符需要2个事件（按下和释放）

            for &c in chunk {
                if c == '\n' || c == '\r' {
                    // 特殊处理换行符
                    let mut down_input: INPUT = unsafe { std::mem::zeroed() };
                    down_input.type_ = INPUT_KEYBOARD;
                    let ki = unsafe { down_input.u.ki_mut() };
                    ki.wVk = VK_RETURN as WORD;
                    ki.dwFlags = 0;

                    let mut up_input: INPUT = unsafe { std::mem::zeroed() };
                    up_input.type_ = INPUT_KEYBOARD;
                    let ki = unsafe { up_input.u.ki_mut() };
                    ki.wVk = VK_RETURN as WORD;
                    ki.dwFlags = KEYEVENTF_KEYUP;

                    inputs.push(down_input);
                    inputs.push(up_input);
                } else {
                    // 普通字符
                    let mut down_input: INPUT = unsafe { std::mem::zeroed() };
                    down_input.type_ = INPUT_KEYBOARD;
                    let ki = unsafe { down_input.u.ki_mut() };
                    ki.wVk = 0;
                    ki.wScan = c as WORD;
                    ki.dwFlags = KEYEVENTF_UNICODE;

                    let mut up_input: INPUT = unsafe { std::mem::zeroed() };
                    up_input.type_ = INPUT_KEYBOARD;
                    let ki = unsafe { up_input.u.ki_mut() };
                    ki.wVk = 0;
                    ki.wScan = c as WORD;
                    ki.dwFlags = KEYEVENTF_UNICODE | KEYEVENTF_KEYUP;

                    inputs.push(down_input);
                    inputs.push(up_input);
                }
            }

            // 批量发送输入事件
            let result = unsafe {
                SendInput(
                    inputs.len() as UINT,
                    inputs.as_mut_ptr(),
                    std::mem::size_of::<INPUT>() as i32,
                )
            };

            if result != inputs.len() as UINT {
                error!(
                    "SendInput失败，只发送了 {} 个输入中的 {}",
                    result,
                    inputs.len()
                );
                return Err(format!("SendInput失败，结果: {}", result));
            }

            chars_processed += chunk.len();

            // 进度反馈 - 每处理10%进度报告一次
            if chars_processed % (total_chars / 10 + 1) == 0 || chars_processed == total_chars {
                let progress = (chars_processed as f64 / total_chars as f64 * 100.0) as u32;
                info!(
                    "输入进度: {}% ({}/{})",
                    progress, chars_processed, total_chars
                );
            }

            // 批次间延迟
            if self.batch_delay_ms > 0 && chars_processed < total_chars {
                thread::sleep(Duration::from_millis(self.batch_delay_ms));
            }

            // 字符间延迟 - 只在慢速模式下使用
            if self.delay_ms > 0 && self.batch_size <= 5 {
                thread::sleep(Duration::from_millis(self.delay_ms));
            }
        }

        // 统计和报告
        let elapsed = start_time.elapsed();
        let chars_per_second = total_chars as f64 / elapsed.as_secs_f64();
        info!(
            "成功模拟输入 {} 个字符，耗时 {:.2}秒，速度 {:.2}字符/秒",
            total_chars,
            elapsed.as_secs_f64(),
            chars_per_second
        );

        Ok(())
    }
}
