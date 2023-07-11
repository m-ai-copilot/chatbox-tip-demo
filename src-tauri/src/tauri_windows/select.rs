pub use super::SELECT_WINDOWS;
use crate::easy_thing::foreground::PlatformForeground;
use crate::AppState;
use crate::APP;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, LogicalPosition, Manager, PhysicalPosition, WindowEvent};
pub const SELECT_WINDOWS_WIDTH: f64 = 320.0;
pub const SELECT_WINDOWS_HEIGHT: f64 = 100.0;



pub fn build_select_windows(
    handle: &AppHandle,
    content: &str,
    window_position_x: f64,
    window_position_y: f64,
) {
    let foreground_handle = PlatformForeground::get_foreground_window();
    tracing::info!(foreground_handle = foreground_handle);
    let state: tauri::State<AppState> = handle.state();
    let _selected = content.to_string();
    if foreground_handle != 0 {
        state
            .foreground_handle
            .store(foreground_handle, Ordering::SeqCst);
    }
    match handle.get_window(SELECT_WINDOWS) {
        Some(window) => {
            tracing::info!("has select window");
            window.unminimize().unwrap();
            if cfg!(target_os = "macos") {
                let _ =
                    window.set_position(LogicalPosition::new(window_position_x, window_position_y));
            } else {
                let _ = window
                    .set_position(PhysicalPosition::new(window_position_x, window_position_y));
            }
            window.show().unwrap();
            window.set_focus().unwrap();
        }
        None => {
            tracing::info!("not found select window");
            println!("select window does not exist, create it !");
            let windows = tauri::WindowBuilder::new(
                handle,
                SELECT_WINDOWS,
                tauri::WindowUrl::App("src/select.html".into()),
            )
            .title("select")
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .maximized(false)
            .skip_taskbar(true)
            .position(window_position_x as f64, window_position_y as f64)
            .inner_size(SELECT_WINDOWS_WIDTH, SELECT_WINDOWS_HEIGHT)
            .focused(true)
            .build()
            .expect("build windows error not happened");
            windows.on_window_event(hide_window_when_lose_focused);

            //windows.set_always_on_top(true).unwrap();
        }
    }
}

fn hide_window_when_lose_focused(event: &WindowEvent) {
    if let WindowEvent::Focused(focused) = event {
        if !focused {
            let handle = APP.get().unwrap();
            if let Some(window) = handle.get_window(SELECT_WINDOWS) {
                //window.set_always_on_top(false).unwrap();
                let _ = window.hide();
            }
        }
    }
}

pub fn hide_select_window() {
    let handle = APP.get().unwrap();
    if let Some(window) = handle.get_window(SELECT_WINDOWS) {
        //window.set_always_on_top(false).unwrap();
        let _ = window.hide();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelectPayload {
    pub label: String,
    pub prompt: String,
    pub selected: String,
}

use tauri::Window;
// fn insert_text_and_send(window: &Window, text: &str) {
//     let script = format!(
//         r#"
//         (function() {{
//             let input = document.querySelector('#message-input');
//             input.value = '{}';  // 设置文本框的值为传入的文本
//             input.focus();
//             //let button = document.querySelector('#send-button');
//             //button.click();
//         }})();
//         "#,
//         text,
//     );
//     window.eval(&script).unwrap();
// }


fn insert_text_and_send(window: &Window, text: &str) {
    let script = format!(
        r#"
      (function() {{
        let input = document.querySelector('#message-input');
        input.focus();
        //let button = document.querySelector('#send-button');
        //button.click();
      }})();"#,
    );
    window.eval(&script).unwrap();
}


use enigo::{Enigo, Key, KeyboardControllable};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::thread;
use std::time::Duration;
use clipboard::{ClipboardContext, ClipboardProvider};



use winapi::um::winuser::{
    CloseClipboard, EmptyClipboard, OpenClipboard,
    SetClipboardData, CF_UNICODETEXT,  GetClipboardData,
};



fn set_clipboard_data(data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard: ClipboardContext = ClipboardProvider::new()?;
    clipboard.set_contents(data.to_owned())?;
    Ok(())
}


fn simulate_paste() {
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Control);

    // 等待按键消息处理
    thread::sleep(Duration::from_millis(100));
    unsafe {
        // 清空剪贴板并关闭
        EmptyClipboard();
        CloseClipboard();
    }
}

fn copy_selected_text() -> Option<String> {
    // 模拟按键：Ctrl + C，复制选定的文本到剪贴板
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('c'));
    enigo.key_up(Key::Control);

    // // 等待按键消息处理
    thread::sleep(Duration::from_millis(100));

    unsafe {
        // 打开剪贴板
        if OpenClipboard(ptr::null_mut()) == 0 {
            println!("Failed to open the clipboard.");
            return None;
        }

        // 获取剪贴板中的文本
        let clipboard_data = GetClipboardData(CF_UNICODETEXT);
        if clipboard_data.is_null() {
            println!("Failed to get clipboard data.");
            CloseClipboard();
            return None;
        }

        // 将剪贴板中的文本转换为 Rust 字符串
        let text_ptr = clipboard_data as *const u16;
        let mut text_length = 0;
        while *text_ptr.offset(text_length) != 0 {
            text_length += 1;
        }

        let text_slice = std::slice::from_raw_parts(text_ptr, text_length as usize);
        let selected_text = OsString::from_wide(text_slice)
            .to_string_lossy()
            .into_owned();

        println!("selected_text: {}", selected_text);

        Some(selected_text)
    }
}

pub fn click_select(handle: &tauri::AppHandle, payload: SelectPayload) -> anyhow::Result<()> {
    print!("tauri_windows.select.rs  click_select {:?}", payload);
    let app_config = crate::app_config::get_app_config().unwrap_or_default();
    let mode = app_config.mode.unwrap_or("快捷提问".to_string());
    println!("mode: {}", mode);
    tracing::info!(mode = mode);

    let selected_text = copy_selected_text().unwrap_or_else(|| "".to_string());
    println!("Selected text: {}", selected_text);
    let combine_text = format!("{} {}", payload.prompt, selected_text);
    println!("combine_text: {}", combine_text);
    
    set_clipboard_data(&combine_text).unwrap();

    let window = handle.get_window("main").unwrap();
    window.unminimize().unwrap();
    // 显示窗口并将其置于最前端
    window.show().unwrap();
    window.set_focus().unwrap();  

    println!("Window is shown and focused");

    // 等待按键消息处理
    thread::sleep(Duration::from_millis(100));
    insert_text_and_send(&window, &selected_text);
    
    // 插入文本到输入框并模拟点击发送按钮
    thread::sleep(Duration::from_millis(100));
    simulate_paste();

    Ok(())
}
