// src/main.rs
mod clipboard;
mod hotkey;
mod input_sim;

use crate::clipboard::ClipboardManager;
use crate::hotkey::{HotkeyManager, MOD_CONTROL, MOD_SHIFT};
use crate::input_sim::{InputSimulator, InputSpeed};
use log::{error, info};
use simple_logger::SimpleLogger;
use std::io::{self, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// 虚拟键码
const VK_V: i32 = 0x56;

enum Action {
    CaptureClipboard,
    SimulateTyping,
    ShowBuffer,
    SetDelay,
    SetSpeedMode,
    CustomParams,
    Exit,
    Hotkey,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .expect("无法初始化日志记录器");

    info!("启动粘贴绕过应用程序（命令行版本）");

    // 创建共享的状态
    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new()));
    let input_simulator = Arc::new(Mutex::new(InputSimulator::with_delay(10))); // 默认10毫秒延迟
    let buffer = Arc::new(Mutex::new(None::<String>));
    let mut hotkey_manager = HotkeyManager::new();

    // 创建一个通道用于动作通知
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    // 注册热键
    hotkey_manager.register(MOD_CONTROL | MOD_SHIFT, VK_V, move || {
        let _ = tx_clone.send(Action::Hotkey);
    })?;

    println!("粘贴绕过工具 (Paste Bypass Tool) - 命令行版本 (Command Line Version)");
    println!("=========================");
    println!("按 Ctrl+Shift+V 触发粘贴绕过 (Press Ctrl+Shift+V to trigger paste bypass)");
    println!("按 Ctrl+C 退出程序 (Press Ctrl+C to exit the program)");
    println!("");

    // 创建用户输入线程
    let tx_user = tx.clone();
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();

    thread::spawn(move || {
        while *running_clone.lock().unwrap() {
            println!("\n选项 (Options):");
            println!("1. 捕获剪贴板 (Capture Clipboard)");
            println!("2. 模拟输入 (Simulate Typing)");
            println!("3. 显示当前缓冲区 (Show Current Buffer)");
            println!("4. 设置按键延迟 (Set Key Delay)");
            println!("5. 设置速度模式 (Set Speed Mode)");
            println!("6. 自定义输入参数 (Custom Input Parameters)");
            println!("7. 退出 (Exit)");

            print!("请选择 (Please select) (1-7): ");
            io::stdout().flush().unwrap();

            let mut choice = String::new();
            if io::stdin().read_line(&mut choice).is_err() {
                continue;
            }

            match choice.trim() {
                "1" => {
                    let _ = tx_user.send(Action::CaptureClipboard);
                }
                "2" => {
                    let _ = tx_user.send(Action::SimulateTyping);
                }
                "3" => {
                    let _ = tx_user.send(Action::ShowBuffer);
                }
                "4" => {
                    let _ = tx_user.send(Action::SetDelay);
                }
                "5" => {
                    let _ = tx_user.send(Action::SetSpeedMode);
                }
                "6" => {
                    let _ = tx_user.send(Action::CustomParams);
                }
                "7" => {
                    let _ = tx_user.send(Action::Exit);
                    break;
                }
                _ => println!("无效的选择，请输入1-7 (Invalid choice, please enter 1-7)"),
            }
        }
    });

    // 克隆共享数据用于主线程
    let clipboard_manager_clone = clipboard_manager.clone();
    let input_simulator_clone = input_simulator.clone();
    let buffer_clone = buffer.clone();

    // 主事件循环
    loop {
        match rx.recv() {
            Ok(action) => {
                match action {
                    Action::CaptureClipboard => {
                        let mut cm = clipboard_manager_clone.lock().unwrap();
                        match cm.capture_clipboard() {
                            Ok(_) => {
                                let text = cm.get_buffer().cloned();
                                drop(cm); // 释放锁

                                if let Some(content) = text {
                                    *buffer_clone.lock().unwrap() = Some(content);
                                    println!("已捕获剪贴板内容 (Clipboard content captured)");
                                }
                            }
                            Err(e) => println!("错误 (Error): {}", e),
                        }
                    }
                    Action::SimulateTyping => {
                        let text = buffer_clone.lock().unwrap().clone();
                        if let Some(text) = text {
                            println!(
                                "将在3秒后开始模拟输入... (Starting simulation in 3 seconds...)"
                            );
                            for i in (1..=3).rev() {
                                println!("{}...", i);
                                thread::sleep(Duration::from_secs(1));
                            }

                            let sim = input_simulator_clone.lock().unwrap();
                            match sim.simulate_typing(&text) {
                                Ok(_) => println!("模拟输入完成 (Typing simulation completed)"),
                                Err(e) => println!("错误 (Error): {}", e),
                            }
                        } else {
                            println!("缓冲区为空，请先捕获剪贴板 (Buffer is empty, please capture clipboard first)");
                        }
                    }
                    Action::ShowBuffer => {
                        let text = buffer_clone.lock().unwrap().clone();
                        if let Some(text) = text {
                            println!("当前缓冲区内容 (Current buffer content):");
                            println!("----------------");
                            println!("{}", text);
                            println!("----------------");
                        } else {
                            println!("缓冲区为空 (Buffer is empty)");
                        }
                    }
                    Action::SetDelay => {
                        print!("输入按键延迟(毫秒) (Enter key delay in ms): ");
                        io::stdout().flush().unwrap();

                        let mut delay = String::new();
                        if io::stdin().read_line(&mut delay).is_err() {
                            continue;
                        }

                        if let Ok(delay_ms) = delay.trim().parse::<u64>() {
                            let mut sim = input_simulator_clone.lock().unwrap();
                            sim.set_delay(delay_ms);
                            println!(
                                "按键延迟已设置为 {} 毫秒 (Key delay set to {} ms)",
                                delay_ms, delay_ms
                            );
                        } else {
                            println!(
                                "无效的输入，请输入一个数字 (Invalid input, please enter a number)"
                            );
                        }
                    }
                    Action::SetSpeedMode => {
                        println!("选择速度模式 (Select speed mode):");
                        println!("1. 慢速 (安全模式，适合严格网站) (Slow - Safe mode for strict websites)");
                        println!("2. 正常 (默认) (Normal - Default)");
                        println!("3. 快速 (适合一般网站) (Fast - For general websites)");
                        println!(
                            "4. 极速 (最快，但可能被检测) (Turbo - Fastest, but may be detected)"
                        );

                        print!("请选择 (Please select) (1-4): ");
                        io::stdout().flush().unwrap();

                        let mut mode_choice = String::new();
                        if io::stdin().read_line(&mut mode_choice).is_err() {
                            continue;
                        }

                        let mut simulator = input_simulator_clone.lock().unwrap();
                        match mode_choice.trim() {
                            "1" => {
                                simulator.set_speed_mode(InputSpeed::Slow);
                                println!("已设置为慢速模式 (Set to slow mode)");
                            }
                            "2" => {
                                simulator.set_speed_mode(InputSpeed::Normal);
                                println!("已设置为正常模式 (Set to normal mode)");
                            }
                            "3" => {
                                simulator.set_speed_mode(InputSpeed::Fast);
                                println!("已设置为快速模式 (Set to fast mode)");
                            }
                            "4" => {
                                simulator.set_speed_mode(InputSpeed::Turbo);
                                println!("已设置为极速模式 (Set to turbo mode)");
                            }
                            _ => println!("无效选择，使用默认正常模式 (Invalid choice, using default normal mode)"),
                        }
                    }
                    Action::CustomParams => {
                        let mut delay_ms = 10;
                        let mut batch_size = 5;
                        let mut batch_delay_ms = 50;

                        print!("输入字符延迟(毫秒，0-100) (Enter character delay in ms, 0-100): ");
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        if io::stdin().read_line(&mut input).is_ok() {
                            if let Ok(val) = input.trim().parse::<u64>() {
                                delay_ms = val;
                            }
                        }

                        print!("输入批处理大小(1-100) (Enter batch size, 1-100): ");
                        io::stdout().flush().unwrap();
                        input.clear();
                        if io::stdin().read_line(&mut input).is_ok() {
                            if let Ok(val) = input.trim().parse::<usize>() {
                                batch_size = val;
                            }
                        }

                        print!("输入批次间延迟(毫秒，0-500) (Enter batch delay in ms, 0-500): ");
                        io::stdout().flush().unwrap();
                        input.clear();
                        if io::stdin().read_line(&mut input).is_ok() {
                            if let Ok(val) = input.trim().parse::<u64>() {
                                batch_delay_ms = val;
                            }
                        }

                        let mut simulator = input_simulator_clone.lock().unwrap();
                        simulator.set_custom_params(delay_ms, batch_size, batch_delay_ms);
                        println!(
                            "已设置自定义参数: 字符延迟={}ms, 批大小={}, 批次延迟={}ms (Custom parameters set: char delay={}ms, batch size={}, batch delay={}ms)",
                            delay_ms, batch_size, batch_delay_ms, delay_ms, batch_size, batch_delay_ms
                        );
                    }
                    Action::Exit => {
                        println!("退出程序 (Exiting program)");
                        // 设置运行标记为false
                        *running.lock().unwrap() = false;
                        break;
                    }
                    Action::Hotkey => {
                        println!("\n[热键触发] 执行粘贴绕过... ([Hotkey triggered] Executing paste bypass...)");

                        // 捕获剪贴板
                        let mut cm = clipboard_manager_clone.lock().unwrap();
                        match cm.capture_clipboard() {
                            Ok(_) => {
                                let text = cm.get_buffer().cloned();
                                drop(cm); // 释放锁

                                if let Some(content) = text {
                                    *buffer_clone.lock().unwrap() = Some(content.clone());
                                    println!("已捕获剪贴板内容 (Clipboard content captured)");

                                    // 短暂延迟，让用户有时间切换窗口
                                    println!("请在3秒内切换到目标窗口... (Please switch to target window within 3 seconds...)");
                                    for i in (1..=3).rev() {
                                        println!("{}...", i);
                                        thread::sleep(Duration::from_secs(1));
                                    }

                                    // 执行模拟输入
                                    let sim = input_simulator_clone.lock().unwrap();
                                    match sim.simulate_typing(&content) {
                                        Ok(_) => {
                                            println!("模拟输入完成 (Typing simulation completed)")
                                        }
                                        Err(e) => println!(
                                            "模拟输入错误 (Typing simulation error): {}",
                                            e
                                        ),
                                    }
                                } else {
                                    println!("缓冲区为空，无法模拟输入 (Buffer is empty, cannot simulate typing)");
                                }
                            }
                            Err(e) => println!("剪贴板捕获错误 (Clipboard capture error): {}", e),
                        }
                    }
                }
            }
            Err(e) => {
                error!("通道错误: {}", e);
                break;
            }
        }
    }

    Ok(())
}
