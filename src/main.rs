use std::fs::File;

use mbrman::MBR;

use crate::download::download_if_needed;

mod download;

fn main() {
    let mut f = File::open("image-to-be-flashed.img").unwrap();
    let mbr = MBR::read_from(&mut f, 512).unwrap();

    println!("Disk signature: {:?}", mbr.header.disk_signature);

    for (i, p) in mbr.iter() {
        if p.is_used() {
            println!(
                "Partition #{}: type = {:?}, size = {} byte, starting lba = {}",
                i,
                p.sys,
                p.sectors * mbr.sector_size,
                p.starting_lba
            )
        }
    }

    let downloads = "https://cdimage.ubuntu.com/releases/22.04/release/";

    download_if_needed(
        false,
        vec![
            get_urls(downloads, "SHA256SUMS").as_str(),
            get_urls(
                downloads,
                "ubuntu-22.04.1-preinstalled-server-arm64+raspi.img.xz",
            )
            .as_str(),
        ],
    )
}

fn get_urls(base: &str, file: &str) -> String {
    format!("{}/{}", base.strip_suffix('/').unwrap().trim(), file.trim())
}
