#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
extern crate core;
#[macro_use(defer)]
extern crate scopeguard;

use std::borrow::Borrow;

use size::Size;

use crate::fetch_media::{download_if_needed, download_if_needed2};

mod compression;
mod configuration;
mod fetch_media;
mod filesystem;
mod partitioning;

// TODO need to verify sha on image
// TODO parameterize download link
// TODO error handling
// TODO most cleanup
/*
steps to reimplement
- if needed
    - expand image file
    - expand filesystem
- mount image to file system
    - do configuration
        - kernel stuff
        - copy config files from embedded fs
        - install packages
        - install k8s related things
        - configure cloud init
        - write up fstab entries

*/
// Mount image

fn main() {
    let image = partitioning::allocate_image("lady_tel_test.img".to_string(), Size::from_gib(3));
    defer!(image.detach());
    println!("{}", image.device.path().unwrap().to_str().unwrap());
    filesystem::create(image.borrow());
    let base_url = "http://os.archlinuxarm.org/os/";
    let image_file = "ArchLinuxARM-rpi-aarch64-latest.tar.gz";
    let hash_file = "ArchLinuxARM-rpi-aarch64-latest.tar.gz.md5";
    download_if_needed(
        false,
        [
            get_urls(base_url, image_file).as_str(),
            get_urls(base_url, hash_file).as_str(),
        ]
        .borrow(),
    );
}

fn get_urls(base: &str, file: &str) -> String {
    format!("{}/{}", base.strip_suffix('/').unwrap().trim(), file.trim())
}
