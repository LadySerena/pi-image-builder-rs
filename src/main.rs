#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
extern crate core;
#[macro_use(defer)]
extern crate scopeguard;

use size::Size;

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
    let device = partitioning::allocate_image("lady_tel_test.img".to_string(), Size::from_gib(3));
    // TODO need to deactivate the logical volume prior to detaching the loop device
    defer!(device.detach().unwrap());
    println!("{}", device.path().unwrap().to_str().unwrap());
}

fn get_urls(base: &str, file: &str) -> String {
    format!("{}/{}", base.strip_suffix('/').unwrap().trim(), file.trim())
}
