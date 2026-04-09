use std::{env, fs, process, thread, time};
use std::os::windows::process::CommandExt;
use std::ptr::null_mut;

use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};

use std::io::Write;
use crate::AppWindow;
use crate::work_file_config;

const PROCESS_NAME: &str = "winws.exe";
const MAX_WAIT_PER_BAT: u64 = 3;


pub fn manager(my_url: String, ui_handle: slint::Weak<AppWindow>) {
    let args: Vec<String> = env::args().collect();

    let url = if args.len() > 1 {
        args[1].clone()
    } else {
        my_url.trim().to_string()
    };

    if url.is_empty() {
        log_to_gui(&ui_handle, "[!] ОШИБКА: ССЫЛКА НЕ УКАЗАНА".to_string());
        return;
    }

    work_file_config::add_url_in_config(&url);

    run_bypasses(url, ui_handle);
}

// проверка на то что у пользователя есть доступ к админке
pub fn is_admin() -> bool {
    let mut handle: HANDLE = null_mut();
    unsafe {
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) == 0 { return false; }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = size_of::<TOKEN_ELEVATION>() as u32;
        let ret = GetTokenInformation(handle, TokenElevation, &mut elevation as *mut _ as *mut _, size, &mut size);
        CloseHandle(handle);
        ret != 0 && elevation.TokenIsElevated != 0
    }
}

// закрытие процесса winws для правильной работы всех батников и что бы они не конфликтовали
fn kill_bypasses() {
    let _ = process::Command::new("taskkill")
        .args(&["/F", "/IM", PROCESS_NAME])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status();
}

// проверка на то url работает под запущенным батником
fn check_address(url: &str) -> bool {
    let client = reqwest::blocking::Client::builder()
        .timeout(time::Duration::from_millis(1500))
        .danger_accept_invalid_certs(true)
        .build().unwrap();

    match client.get(url).send() {
        Ok(res) => res.status().as_u16() < 400,
        Err(_) => false,
    }
}

// запуск основной логики программы
fn run_bypasses(target_url: String, ui_handle: slint::Weak<AppWindow>) {
    let mut core_path = env::current_exe().unwrap();
    core_path.pop();
    core_path.push("core");

    log_to_gui(&ui_handle, format!("[*] Цель: {}", target_url));
    kill_bypasses();

    let paths = match fs::read_dir(&core_path) {
        Ok(p) => p,
        Err(_) => {
            log_to_gui(&ui_handle, "[!] ОШИБКА: Папка 'core' не найдена рядом с EXE".to_string());
            return;
        }
    };

    let bat_files: Vec<_> = paths
        .filter_map(|r| r.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            p.extension().map_or(false, |ext| ext == "bat") && name != "start.bat"
        })
        .collect();

    for bat in bat_files {
        let bat_name = bat.file_name().unwrap().to_string_lossy();
        log_to_gui(&ui_handle, format!("[*] Пробуем: {}...", bat_name));
        std::io::stdout().flush().unwrap();

        let full_path = fs::canonicalize(&bat).expect("Не удалось получить путь");
        let clean_path = full_path.to_string_lossy().replace(r"\\?\", "");

        let mut child = process::Command::new("cmd")
            .args(&["/C", &clean_path])
            .current_dir(&core_path)
            .creation_flags(0x00000010)
            .spawn()
            .expect("Ошибка запуска bat");

        let start_time = time::Instant::now();
        let mut success = false;

        while start_time.elapsed().as_secs() < MAX_WAIT_PER_BAT {
            if check_address(&target_url) {
                success = true;
                break;
            }
            thread::sleep(time::Duration::from_millis(MAX_WAIT_PER_BAT * 100));
        }

        if success {
            log_to_gui(&ui_handle, "\n[+] РАБОТАЕТ!".to_string());

            std::mem::forget(child);
            return;
        } else {
            log_to_gui(&ui_handle, "нет доступа.".to_string());
            let _ = child.kill();
            kill_bypasses();
            thread::sleep(time::Duration::from_millis(MAX_WAIT_PER_BAT * 100));
        }
    }

    log_to_gui(&ui_handle, "\n[!] ОШИБКА: НЕТ АКТИВНЫХ ОБХОДОВ".to_string());
}

// логирование
fn log_to_gui(ui_handle: &slint::Weak<AppWindow>, msg: String) {
    let ui_handle = ui_handle.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let current = ui.get_status_log();
            ui.set_status_log(format!("{}\n{}", current, msg).into());
        }
    });
}

