use std::fs;

use sys_mount::{Mount, SupportedFilesystems, Unmount, UnmountFlags};

use crate::partitioning::ImageInfo;

pub fn mount(info: &ImageInfo) {
    let supported = SupportedFilesystems::new().unwrap();

    fs::create_dir_all("./fake-root/boot").unwrap();
    let root = Mount::builder()
        .fstype("ext4")
        .mount(info.root_path(), "./fake-root")
        .unwrap()
        .into_unmount_drop(UnmountFlags::DETACH);
}
