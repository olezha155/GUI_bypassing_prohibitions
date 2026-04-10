use crate::merge_sort;

use std::{env, fs};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use slint::SharedString;

// берет переданные ссылки и добавляет их в конфиг в виде доменов
pub fn add_domain_in_config(url: &str) {
    let mut core_path = env::current_exe().unwrap();
    core_path.pop();
    core_path.push("core");
    core_path.push("lists");
    core_path.push("list-general.txt");

    let file = File::open(&core_path).expect("ERROR: open file");
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = Vec::with_capacity(60);
    lines.extend(reader.lines().map_while(Result::ok));

    merge_sort::merge_sort(&mut lines);

    let mut lines_domain: Vec<String> = Vec::with_capacity(5);
    lines_domain.extend(url.lines().map(|s| s.to_string()));

    for line_domain in lines_domain {
        let domain = get_domain_by_url(&line_domain);

        match lines.binary_search(&domain) {
            Ok(_) => {}
            Err(_) => {
                let mut file = OpenOptions::new().append(true).open(&core_path).unwrap();

                writeln!(file, "{}", domain).expect("ERROR: write in file");
            }
        }
    }
}

// получение домена по переданной ссылке
fn get_domain_by_url(url: &str) -> String {
    let mut new_url = url.trim();

    if let Some(pos) = new_url.find("//") {
        new_url = &new_url[pos + 2..];
    }

    if let Some(index) = new_url.find('/') {
        new_url = &new_url[..index];
    }

    new_url.to_string()
}

pub fn get_bat_files() -> Vec<SharedString> {
    let mut core_path = env::current_exe().unwrap();
    core_path.pop();
    core_path.push("core");

    let mut bat_files: Vec<SharedString> = Vec::with_capacity(20);
    bat_files.push("None".into());

    if let Ok(entries) = fs::read_dir(core_path) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("bat") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if file_name != "service.bat" {
                        bat_files.push(file_name.into());
                    }
                }
            }
        }
    }

    bat_files
}
