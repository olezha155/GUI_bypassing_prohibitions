use crate::merge_sort;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Write, BufReader};

// добавляет домен если его нет в файле конфигов
pub fn add_url_in_config(url: &str) {
    let mut core_path = env::current_exe().unwrap();
    core_path.pop();
    core_path.push("core");
    core_path.push("lists");
    core_path.push("list-general.txt");

    let file = File::open(&core_path).expect("ERROR: open file");
    let reader = BufReader::new(file);

    // читаем строки, фильтруем ошибки и собираем в вектор
    let mut lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .collect();

    merge_sort::merge_sort(&mut lines);

    let domain = get_domain_by_url(url);
    match lines.binary_search(&domain) {
        Ok(_) => {}
        Err(_) => {
            let mut file = OpenOptions::new()
                .append(true)
                .open(core_path).unwrap();

            writeln!(file, "{}", domain).expect("ERROR: write in file");
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
