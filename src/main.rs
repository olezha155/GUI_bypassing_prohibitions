// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod manager;
mod merge_sort;
mod work_file_config;

use std::error::Error;

use slint::ModelRc;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::process::CommandExt;
use std::ptr::null_mut;
use std::{env, process};
use winapi::um::shellapi::ShellExecuteW;
use winapi::um::winuser::SW_HIDE;

use crate::manager::SETTINGS;

use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Write;

slint::include_modules!();

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    dark_mode: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // проверка, что у пользователя есть админка
    if !manager::is_admin() {
        let exe_path = std::env::current_exe()?;
        let file: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();
        let operation: Vec<u16> = "runas\0".encode_utf16().collect();

        unsafe {
            ShellExecuteW(
                null_mut(),
                operation.as_ptr(),
                file.as_ptr(),
                null_mut(),
                null_mut(),
                SW_HIDE,
            );
        }
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
                .args(&["/C", "start", "/b", &conf])
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
        std::thread::spawn(move || work_file_config::add_domain_in_config(url.as_str()));
    });

    // установка времени подключения к сайту
    {
        let ui_handle = ui.as_weak();

        if let Some(ui) = ui_handle.upgrade() {
            ui.set_time_connect_to_site("5".into());
        }

        ui.on_exit_from_settings({
            move |text| {
                let mut settings = SETTINGS.write().unwrap();

                settings.max_wait_per_bat = text.parse::<u64>().unwrap();
            }
        });
    }

    {
        let config_name = "gui_app_prefs";

        // 1. Загружаем конфиг
        let saved_cfg: AppConfig = confy::load(config_name, None).unwrap_or_default();

        ui.set_dark_mode(saved_cfg.dark_mode);

        ui.on_save_config(move |is_dark| {
            // Rust просто меняет свойство окна, а Slint сам обновит Theme через <=>
            let _ = confy::store(config_name, None, AppConfig { dark_mode: is_dark });
        });
    }

    // вывод конфига пользователя
    {
        let ui_handle_for_conf = ui.as_weak();

        ui.on_print_config(move || {
            let win = UpReadConfAll::new().unwrap();

            let mut core_path = env::current_exe().unwrap();
            core_path.pop();
            core_path.extend(["core", "lists", "list-general.txt"]);

            let content = std::fs::read_to_string(&core_path).unwrap_or_else(|_| String::new());

            win.set_all_config(slint::SharedString::from(&content));

            let path_clone = core_path.clone();
            let win_weak = win.as_weak();

            win.on_save_config(move || {
                if let Some(win) = win_weak.upgrade() {
                    let current_text = win.get_all_config();

                    if let Ok(mut file) = File::create(&path_clone) {
                        let _ = file.write_all(current_text.as_bytes());
                    }

                    win.hide().unwrap();
                }
            });

            if let Some(main_ui) = ui_handle_for_conf.upgrade() {
                win.set_dark_mode(main_ui.get_dark_mode());
            }

            win.show().unwrap();
            std::mem::forget(win);
        });
    }

    {
        // Создаем клон для использования внутри замыкания
        let ui_handle_for_help = ui.as_weak();

        ui.on_open_help(move || {
            let help_text = std::fs::read_to_string("./help.txt").unwrap_or_else(|_| {
                "Ошибка: файл help.txt не найден в директории приложения.".to_string()
            });

            let win = Help::new().unwrap();

            // Используем клон хендла
            if let Some(main_ui) = ui_handle_for_help.upgrade() {
                win.set_dark_mode(main_ui.get_dark_mode());
            }

            win.set_help_content(help_text.into());
            win.show().unwrap();

            // Предотвращаем уничтожение окна сразу после выхода из области видимости
            std::mem::forget(win);
        });
    }

    ui.run()?;
    manager::kill_bypasses();
    Ok(())
}
