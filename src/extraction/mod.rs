use flate2::read::GzDecoder;
use std::fs;
use std::fs::File;
use std::path::Path;
use tar::Archive;

pub fn tarball(path: &Path) {
    let tar_gz = File::open(path).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    fs::create_dir_all("./mnt/boot").unwrap();
    // TODO https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html
    // TODO https://crates.io/crates/sys-mount
    // I still need to mount the loop devices somewhere
    archive.unpack("./mnt").unwrap();
}
