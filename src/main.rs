// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod manager;
mod work_file_config;
mod merge_sort;

use std::error::Error;

use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winuser::SW_HIDE;
use std::os::windows::ffi::OsStrExt;
use std::{env, process};
use std::os::windows::process::CommandExt;
use std::ptr::null_mut;
use slint::ModelRc;

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

    let items = work_file_config::get_bat_files();
    let model = ModelRc::new(slint::VecModel::from(items));
    ui.set_combo_options(model);

    // активация батников
    ui.on_activate_clicked(move |url, config| {
        let ui_handle_for_thread = ui_handle.clone();
        let url_str = url.to_string();
        let conf = config.to_string();

        if conf.eq("Auto") {
            std::thread::spawn(move || {
                manager::manager(url_str, ui_handle_for_thread);
            });
        } else {
            manager::kill_bypasses();
            let mut core_path = env::current_exe().unwrap();
            core_path.pop();
            core_path.push("core");

            let child = process::Command::new("cmd")
                .args(&["/C", &conf])
                .current_dir(&core_path)
                .creation_flags(0x08000000)
                .spawn()
                .expect("Ошибка запуска bat");

            std::mem::forget(child);
            manager::log_to_gui(&ui_handle, format!("\n[+] Подключено к {}", conf));
        }
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
