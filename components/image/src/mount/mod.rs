use std::fs;
use std::path::{Path, PathBuf};

use sys_mount::{Mount, MountFlags, Mounts, Unmount, UnmountFlags};

use crate::extraction::tarball;
use crate::partitioning::ImageInfo;

pub fn mount(info: &ImageInfo, source: &Path) -> Mounts {
    let root_path = Path::new("./fake-root");

    fs::create_dir_all(root_path).unwrap();
    let root = Mount::builder()
        .fstype("ext4")
        .mount(info.root_path(), root_path)
        .unwrap()
        .into_unmount_drop(UnmountFlags::DETACH);

    let boot_path = append_to_root(root_path, "boot");

    fs::create_dir(boot_path.as_path()).unwrap();

    let image_boot = info.boot_path();

    let boot = Mount::builder()
        .fstype("vfat")
        .mount(image_boot.as_path(), boot_path.as_path())
        .unwrap()
        .into_unmount_drop(UnmountFlags::DETACH);

    tarball(source, root_path);

    let proc_path = append_to_root(root_path, "proc");

    let proc = Mount::builder()
        .fstype("proc")
        .mount("/proc", proc_path)
        .unwrap()
        .into_unmount_drop(UnmountFlags::DETACH);

    let sys_path = append_to_root(root_path, "sys");

    let sys = Mount::builder()
        .fstype("sysfs")
        .mount("/sys", sys_path)
        .unwrap()
        .into_unmount_drop(UnmountFlags::DETACH);

    let dev_path = append_to_root(root_path, "dev");

    let dev = Mount::builder()
        .fstype("devtmpfs")
        .flags(MountFlags::BIND)
        .mount("/dev", dev_path)
        .unwrap()
        .into_unmount_drop(UnmountFlags::DETACH);

    Mounts(vec![root, boot, proc, sys, dev])
}

fn append_to_root(root: &Path, data: &str) -> PathBuf {
    let output: PathBuf = [root, Path::new(data)].iter().collect();
    output
}
