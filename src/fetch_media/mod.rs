use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use reqwest::blocking::{Client, ClientBuilder};
use reqwest::Url;

// pub struct Download(Url, Url);
//
// impl Download {
//     pub fn new(media: &str, hash: &str) -> Download {
//         let media_parsed = Url::parse(media).unwrap();
//         let hash_parsed = Url::parse(hash).unwrap();
//         Download {
//             0: media_parsed,
//             1: hash_parsed,
//         }
//     }
// //todo convert to return a slice
//     pub fn get_filenames(&self) -> Sli<String> {
//         (
//             self.0.path_segments().unwrap().last().unwrap().to_string(),
//             self.1.path_segments().unwrap().last().unwrap().to_string(),
//         )
//     }
// }

pub fn download_if_needed(force_overwrite: bool, urls: &[&str]) {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(600))
        .build()
        .unwrap();
    // the intent is to get the file names for the hash verification
    let files: Vec<String> = urls
        .iter()
        .map(|url_string| {
            let parsed_url = Url::parse(url_string).unwrap();
            let file_name = parsed_url.path_segments().unwrap().last().unwrap();
            let output = Path::new(file_name);
            if force_overwrite || !check_if_file_exists(output) {
                download_file(parsed_url.clone(), output, client.clone());
            }
            file_name.to_string()
        })
        .collect();
}

pub fn download_if_needed2(force_overwrite: bool, download: Download) {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(600))
        .build()
        .unwrap();

    let files = download.get_filenames();

    // for url in urls {
    //     let parsed_url = Url::parse(url).unwrap();
    //     let file_name = parsed_url.path_segments().unwrap().last().unwrap();
    //     let output = Path::new(file_name);
    //     if force_overwrite || !check_if_file_exists(output) {
    //         download_file(parsed_url.clone(), output, client.clone());
    //     }
    // }
}

fn download_file(url: Url, file_name: &Path, client: Client) {
    let response = client.get(url).send().unwrap();

    if response.status().is_success() {
        let content = response.bytes().unwrap();
        let mut file = File::create(file_name).unwrap();
        file.write_all(content.as_ref()).unwrap();
    }
}

fn check_if_file_exists(path: &Path) -> bool {
    File::open(path).is_ok()
}
