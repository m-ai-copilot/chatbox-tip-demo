#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod easy_thing;
mod app_config;
mod command;
mod select;
#[cfg(not(target_os = "macos"))]
mod task;
mod tauri_windows;
mod utils;

use app_config::AppConfig;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicIsize;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tauri::Manager;
use tauri::WindowEvent;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub static APP: OnceCell<AppHandle> = OnceCell::new();

pub const CHATBOX_MAIN_WINDOWS: &str = "chatbox_main_windows";
pub struct AppState {
    pub selected_content: Arc<RwLock<String>>,
    pub foreground_handle: AtomicIsize,
    runtime: Runtime,
    pub auto_input_sender: OnceCell<UnboundedSender<String>>,
    pub screen_size: (f64, f64), // (width, height)
    pub enable_select: AtomicBool,
}

impl AppState {
    pub fn new(app_config: &AppConfig, runtime: Runtime, screen_size: (f64, f64)) -> Self {
        Self {
            selected_content: Arc::new(RwLock::new(String::new())),
            foreground_handle: AtomicIsize::new(0),
            runtime,
            auto_input_sender: OnceCell::new(),
            screen_size,
            enable_select: AtomicBool::new(app_config.enable_select.unwrap_or(true)),
        }
    }

    pub fn spawn_future<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<()>
    where
        F: Send + 'static + FnOnce() -> (),
    {
        self.runtime.spawn(async move { task() })
    }

    pub fn spawn_delay_task<F>(&self, future: F, delay_time: Duration) -> JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(async move {
            tokio::time::sleep(delay_time).await;
            future.await
        })
    }
}

fn main() {
  tracing_subscriber::registry().with(fmt::layer()).init();
  // get screen size
  let screen_size = crate::utils::get_screen_size().unwrap_or((1920.0, 1080.0));

  #[allow(unused_mut)]
  let mut context = tauri::generate_context!();

  let app_config = crate::app_config::get_app_config().unwrap_or_default();  

  #[cfg(not(target_os = "macos"))]
  let mut builder = tauri::Builder::default().plugin(tauri_plugin_store::Builder::default().build());

  builder = builder.invoke_handler(tauri::generate_handler![
      command::get_selected_content,
      command::set_size,
      command::run_auto_input,
      command::send_auto_input_value,
      command::run_quick_answer,
      command::run_chat_mode,
      command::close_window,
      command::open_setting_window,
      command::copy_select_content,
      command::update_shortcut,
      command::update_app_config,
      command::get_selected_content_from_cache,
      command::hide_select_window,
      command::trigger_select_click,
  ]);

  builder
  //.with_label(CHATBOX_MAIN_WINDOWS) // 添加标签到主窗口
  .setup(move |app| {
      tracing::info!(start = true);
      APP.get_or_init(|| app.handle());
      let app_handle = app.handle();
      app_handle.manage(AppState::new(
          &app_config,
          tokio::runtime::Runtime::new().expect("build tokio runtime error"),
          screen_size,
      ));

      // 注册全局快捷键
      //let _ = shortcut::ShortcutRegister::register_shortcut(&app_handle);
      #[cfg(not(target_os = "macos"))]
      task::register_task(&app_handle);
      Ok(())
  })
  //.system_tray(system_tray())
  //.on_system_tray_event(handle_click_system_tray)
  .build(context)
  .expect("error while running tauri application")
  //.plugin(tauri_plugin_store::Builder::default().build())
  .run(|app_handle, event| match event {
      tauri::RunEvent::WindowEvent { label, event, .. } => {
          if label == crate::tauri_windows::SELECT_WINDOWS
              //|| label == crate::tauri_windows::search::SEARCH_WINDOWS
          {
              if let WindowEvent::Focused(focused) = event {
                  tracing::info!(label = label, focused = focused);
                  if !focused {
                      if let Some(window) = app_handle.get_window(&label) {
                          let _ = window.hide();
                      }
                  }
              }
          }
      }
      _ => {}
  });


}
