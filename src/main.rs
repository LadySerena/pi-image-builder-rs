#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
#[macro_use(defer)]
extern crate scopeguard;
extern crate core;

use size::Size;

mod compression;
mod configuration;
mod fetch_media;
mod loop_shenanigans;
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
    partitioning::allocate_image("lady_tel_test.img".to_string(), Size::from_gib(3));
}

fn get_urls(base: &str, file: &str) -> String {
    format!("{}/{}", base.strip_suffix('/').unwrap().trim(), file.trim())
}
