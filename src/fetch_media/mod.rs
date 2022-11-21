use md5::{Digest, Md5};
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::Url;
use std::borrow::Borrow;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Duration;

pub struct Download(Url, Url);

impl Download {
    pub fn new(media: &str, hash: &str) -> Download {
        let media_parsed = Url::parse(media).unwrap();
        let hash_parsed = Url::parse(hash).unwrap();
        Download(media_parsed, hash_parsed)
    }

    pub fn get_filenames(&self) -> Vec<(Url, String)> {
        vec![
            (
                self.0.clone(),
                self.0.path_segments().unwrap().last().unwrap().to_string(),
            ),
            (
                self.1.clone(),
                self.1.path_segments().unwrap().last().unwrap().to_string(),
            ),
        ]
    }
}

pub fn download_if_needed(force_overwrite: bool, download: &Download) {
    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(600))
        .build()
        .unwrap();

    let files = download.get_filenames();

    for info in &files {
        let output = Path::new(info.1.as_str());
        if force_overwrite || !check_if_file_exists(output) {
            download_file(info.0.clone(), output, client.borrow());
        }
    }

    // parse upstream hash file

    let hash_path = Path::new(files[1].1.as_str());
    let mut hash_handle = File::open(hash_path).unwrap();
    let mut hash_content = String::new();
    hash_handle.read_to_string(&mut hash_content).unwrap();
    let hash_split = hash_content
        .split(' ')
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>();
    let hash_actual = hash_split[0].as_str();

    // get file hash

    let image_path = Path::new(files[0].1.as_str());
    let mut image_handle = File::open(image_path).unwrap();
    let mut hasher = Md5::new();
    let n = io::copy(&mut image_handle, &mut hasher).unwrap();
    assert_eq!(n, image_handle.metadata().unwrap().size());
    let image_hash = hasher.finalize();
    let image_hash_string = format!("{image_hash:x}");
    assert_eq!(image_hash_string.as_str(), hash_actual);
}

fn download_file(url: Url, file_name: &Path, client: &Client) {
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
