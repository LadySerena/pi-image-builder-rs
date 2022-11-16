use std::fs::{File, OpenOptions};
use std::io::Write;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::partitioning::partition::create_partition_tables;
use loopdev::{LoopControl, LoopDevice};

use size::Size;

mod lvm;
mod partition;

pub struct ImageInfo {
    pub vg_name: String,
    pub lv_name: String,
    pub device: LoopDevice,
}

impl ImageInfo {
    pub fn detach(&self) {
        lvm::deactivate_logical_volume(self.vg_name.as_str(), self.lv_name.as_str()).unwrap();
        lvm::close_lvm();
        self.device.detach().unwrap();
    }
}

pub fn allocate_image(image_path: String, size: Size) -> ImageInfo {
    // needs to do the following

    let path = Rc::new(PathBuf::from(image_path));
    let mut file_handle = allocate_file(path.as_ref(), size);
    create_partition_tables(&mut file_handle, Size::from_mebibytes(250));
    file_handle.flush().unwrap();
    let backing_device = Rc::new(setup_loop(path.as_ref()));

    let volumes = lvm::logical_volume_creation(backing_device.as_ref());
    let vg_name = volumes.0;
    let lv_name = volumes.1;

    ImageInfo {
        vg_name,
        lv_name,
        device: Rc::try_unwrap(backing_device).unwrap(),
    }
}

fn allocate_file(path: &Path, size: Size) -> File {
    let handle = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    handle.set_len(size.bytes() as u64).unwrap();
    handle
}

fn setup_loop(path: &Path) -> LoopDevice {
    let lc = LoopControl::open().unwrap();
    let device = lc.next_free().unwrap();
    device
        .with()
        .part_scan(true)
        .attach(path.to_str().unwrap())
        .unwrap();
    device
}
