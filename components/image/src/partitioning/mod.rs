use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use loopdev::{LoopControl, LoopDevice};
#[cfg(test)]
use mockall::automock;
use size::Size;

use crate::partitioning::partition::create_partition_tables;

mod lvm;
mod partition;

#[cfg_attr(test, automock)]
pub trait ImageInfo {
    fn detach(&self);
    fn root_path(&self) -> PathBuf;
    fn boot_path(&self) -> PathBuf;
}

pub struct RuntimeImageInfo {
    pub vg_name: String,
    pub lv_name: String,
    pub device: LoopDevice,
}

impl ImageInfo for RuntimeImageInfo {
    fn detach(&self) {
        lvm::deactivate_logical_volume(self.vg_name.as_str(), self.lv_name.as_str()).unwrap();
        lvm::close_lvm();
        self.device.detach().unwrap();
    }

    fn root_path(&self) -> PathBuf {
        let temp_buff = format!("/dev/{}/{}", self.vg_name, self.lv_name);
        PathBuf::from(temp_buff)
    }

    fn boot_path(&self) -> PathBuf {
        // TODO replace this clone but the data shouldn't change so ehhhh?
        let path = self.device.path().unwrap();
        let foo = format!("{}{}", path.to_str().unwrap(), "p1");
        PathBuf::from(foo)
    }
}

pub fn allocate_image(image_path: String, size: Size) -> RuntimeImageInfo {
    // needs to do the following

    let path = Rc::new(PathBuf::from(image_path));
    let mut file_handle = allocate_file(path.as_ref(), size);
    create_partition_tables(&mut file_handle, Size::from_mebibytes(400));
    file_handle.flush().unwrap();
    let backing_device = Rc::new(setup_loop(path.as_ref()));

    let volumes = lvm::logical_volume_creation(backing_device.as_ref());
    let vg_name = volumes.0;
    let lv_name = volumes.1;

    RuntimeImageInfo {
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
