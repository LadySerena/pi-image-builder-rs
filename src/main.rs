use std::fs::File;

use crate::compression::xz_decompress;
use mbrman::MBR;

use crate::fetch_media::download_if_needed;

mod compression;
mod fetch_media;
mod loop_shenanigans;
mod lvm;

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
    let image_file = "ubuntu-22.04.1-preinstalled-server-arm64+raspi.img.xz";
    let extracted_image_file = image_file.strip_suffix(".xz").unwrap();

    // download_if_needed(
    //     false,
    //     vec![
    //         get_urls(downloads, "SHA256SUMS").as_str(),
    //         get_urls(downloads, image_file).as_str(),
    //     ],
    // );
    //
    // xz_decompress(image_file.to_string(), extracted_image_file.to_string());

    loop_shenanigans::map_image_to_loop_device(extracted_image_file.to_string());
}

fn get_urls(base: &str, file: &str) -> String {
    format!("{}/{}", base.strip_suffix('/').unwrap().trim(), file.trim())
}
