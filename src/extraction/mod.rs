use flate2::read::GzDecoder;
use std::fs;
use std::fs::File;
use std::path::Path;
use tar::Archive;

pub fn tarball(tar_path: &Path, root_path: &Path) {
    let tar_gz = File::open(tar_path).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(root_path).unwrap();
}
