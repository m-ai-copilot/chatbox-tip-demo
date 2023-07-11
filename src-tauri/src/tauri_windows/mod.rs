#[cfg(not(target_os = "macos"))]
pub mod select;

pub const SELECT_WINDOWS: &str = "select_windows";
