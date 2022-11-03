use std::fs::File;
use std::io::Write;
use std::path::Path;

use reqwest::Url;

pub fn download_if_needed(force_overwrite: bool, urls: Vec<&str>) {
    for url in urls {
        let parsed_url = Url::parse(url).unwrap();
        let file_name = parsed_url.path_segments().unwrap().last().unwrap();
        let output = Path::new(file_name);
        if force_overwrite || !check_if_file_exists(output) {
            download_file(parsed_url.clone(), output);
        }
    }
}

fn download_file(url: Url, file_name: &Path) {
    let response = reqwest::blocking::get(url).unwrap();

    if response.status().is_success() {
        let content = response.bytes().unwrap();
        let mut file = File::create(file_name).unwrap();
        file.write_all(content.as_ref()).unwrap();
    }
}

fn check_if_file_exists(path: &Path) -> bool {
    File::open(path).is_ok()
}
