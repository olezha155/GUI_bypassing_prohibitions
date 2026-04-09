// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod manager;

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    // 1. Создаем слабую ссылку ЗДЕСЬ
    let ui_handle = ui.as_weak();

    ui.on_activate_clicked(move |url| {
        // 2. Клонируем её для передачи в поток
        let ui_handle_for_thread = ui_handle.clone();
        let url_str = url.to_string();

        std::thread::spawn(move || {
            // Передаем в менеджер клонированную слабую ссылку
            manager::manager(url_str, ui_handle_for_thread);
        });
    });

    ui.run()?;
    Ok(())
}
