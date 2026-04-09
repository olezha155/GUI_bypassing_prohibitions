// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod manager;
mod work_file_config;
mod merge_sort;

use std::error::Error;

use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winuser::SW_HIDE;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // проверка, что у пользователя есть админка
    if !manager::is_admin() {
        let exe_path = std::env::current_exe()?;
        let file: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();
        let operation: Vec<u16> = "runas\0".encode_utf16().collect();

        unsafe { ShellExecuteW(null_mut(), operation.as_ptr(), file.as_ptr(), null_mut(), null_mut(), SW_HIDE); }
        return Ok(());
    }

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // активация батников
    ui.on_activate_clicked(move |url| {
        let ui_handle_for_thread = ui_handle.clone();
        let url_str = url.to_string();

        std::thread::spawn(move || {
            manager::manager(url_str, ui_handle_for_thread);
        });
    });

    // добавления домена в конфиги
    ui.on_config_add_clicked(move |url| {
        std::thread::spawn(move || {
            work_file_config::add_domain_in_config(url.as_str())
        });
    });

    ui.run()?;
    manager::kill_bypasses();
    Ok(())
}
