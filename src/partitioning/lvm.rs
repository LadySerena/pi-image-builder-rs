use std::borrow::Borrow;
use std::error::Error as StdError;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::ptr;

use loopdev::LoopDevice;
use lvm_rs::{bd_lvm_lvdeactivate, gchar, BDLVMVGdata};
use size::Size;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {}

#[derive(Clone)]
pub struct LVMVGData {
    pub name: String,
    pub uuid: String,
    // u64 is bytes except for extent_count, free_count, pv_count
    pub size: u64,
    pub free: u64,
    pub extent_size: u64,
    pub extent_count: u64,
    pub free_count: u64,
    pub pv_count: u64,
}

#[derive(Clone)]
pub struct LVMLVData {
    pub lv_name: String,
    pub vg_name: String,
    pub uuid: String,
    // u64 is bytes except for extent_count, free_count, pv_count
    pub size: u64,
    pub attributes: String,
    pub segment_type: String,
    pub origin: String,
    // there is more data for thin and cached logical volumes but I'm only using linear
}

const BYTE_TO_MEBIBYTE_FACTOR: u64 = 1024 * 1024;
const BYTE_TO_GIBIBYTE_FACTOR: u64 = BYTE_TO_MEBIBYTE_FACTOR * 1024;

pub fn logical_volume_creation(device: &LoopDevice) -> (String, String) {
    let init = init_lvm();
    assert!(init, "could not initialize lvm");
    let path = get_partition_path(device).unwrap();
    pv_create(path.as_str()).unwrap();
    let volume_group_name = vg_create(path.as_str()).unwrap();
    let volume_group_data = get_volume_group(volume_group_name).unwrap();
    let logical_volume_name = lv_create(volume_group_data.borrow()).unwrap();
    (
        volume_group_name.to_string(),
        logical_volume_name.to_string(),
    )
}

pub fn deactivate_logical_volume(vg_name: &str, lv_name: &str) -> Result<(), Error> {
    let c_lv_name = CString::new(lv_name).unwrap();
    let c_vg_name = CString::new(vg_name).unwrap();
    unsafe {
        let error = ptr::null_mut();
        let ok = bd_lvm_lvdeactivate(
            c_vg_name.as_ptr(),
            c_lv_name.as_ptr(),
            ptr::null_mut(),
            error,
        );
        if ok == 0 {
            let slice = CStr::from_ptr((*(*error)).message);
            let ret = Error {
                message: String::from_utf8_lossy(slice.to_bytes()).to_string(),
            };
            Err(ret)
        } else {
            Ok(())
        }
    }
}

fn init_lvm() -> bool {
    let dependency_check = unsafe {
        let output = lvm_rs::bd_lvm_check_deps();
        output != 0
    };

    if !dependency_check {
        return false;
    }

    unsafe {
        let output = lvm_rs::bd_lvm_init();
        output != 0
    }
}

fn pv_create(partition_path: &str) -> Result<(), Error> {
    let c_partition = CString::new(partition_path).unwrap();

    unsafe {
        let error = ptr::null_mut();
        let create_output =
            lvm_rs::bd_lvm_pvcreate(c_partition.as_ptr(), 0, 0, ptr::null_mut(), error);
        if create_output == 0 {
            let slice = CStr::from_ptr((*(*error)).message);
            let ret = Error {
                message: String::from_utf8_lossy(slice.to_bytes()).to_string(),
            };
            Err(ret)
        } else {
            Ok(())
        }
    }
}

fn vg_create(partition_path: &str) -> Result<&str, Error> {
    let volume_group_name = "rootvg";
    let c_volume_group_name = CString::new(volume_group_name).unwrap();
    let c_partition = CString::new(partition_path).unwrap();

    unsafe {
        let error = ptr::null_mut();
        let mut pv_list: *mut *const gchar = ptr::null_mut();
        pv_list = &mut c_partition.as_ptr();
        let create_output = lvm_rs::bd_lvm_vgcreate(
            c_volume_group_name.as_ptr(),
            pv_list,
            0,
            ptr::null_mut(),
            error,
        );
        if create_output == 0 {
            let slice = CStr::from_ptr((*(*error)).message);
            let ret = Error {
                message: String::from_utf8_lossy(slice.to_bytes()).to_string(),
            };
            Err(ret)
        } else {
            Ok(volume_group_name)
        }
    }
}

fn get_volume_group(volume_group_name: &str) -> Option<LVMVGData> {
    let c_volume_group_name = CString::new(volume_group_name).unwrap();
    unsafe {
        let error = ptr::null_mut();
        let output = lvm_rs::bd_lvm_vginfo(c_volume_group_name.as_ptr(), error);

        if !error.is_null() {
            return None;
        }

        let c_name = CStr::from_ptr((*output).name);
        let name = String::from_utf8_lossy(c_name.to_bytes()).to_string();
        let c_uuid = CStr::from_ptr((*output).uuid);
        let uuid = String::from_utf8_lossy(c_uuid.to_bytes()).to_string();
        let size = (*output).size;
        let free = (*output).free;
        let extent_size = (*output).extent_size;
        let extent_count = (*output).extent_count;
        let free_count = (*output).free_count;
        let pv_count = (*output).pv_count;
        let safe = LVMVGData {
            name,
            uuid,
            size,
            free,
            extent_size,
            extent_count,
            free_count,
            pv_count,
        };
        Some(safe)
    }
}

fn lv_create(volume_group: &LVMVGData) -> Result<&str, Error> {
    let logical_name = "rootlv";
    let c_logical_name = CString::new(logical_name).unwrap();
    let volume_group_name: &str = volume_group.name.borrow();
    let c_volume_group = CString::new(volume_group_name).unwrap();
    let volume_type = "linear";
    let c_volume_type = CString::new(volume_type).unwrap();

    unsafe {
        let error = ptr::null_mut();
        let vg_list: *const gchar = c_volume_group.as_ptr();
        let lv_name: *const gchar = c_logical_name.as_ptr();
        let type_name: *const gchar = c_volume_type.as_ptr();
        let create_output = lvm_rs::bd_lvm_lvcreate(
            vg_list,
            lv_name,
            volume_group.size,
            type_name,
            ptr::null_mut(),
            ptr::null_mut(),
            error,
        );
        if create_output == 0 {
            let slice = CStr::from_ptr((*(*error)).message);
            let ret = Error {
                message: String::from_utf8_lossy(slice.to_bytes()).to_string(),
            };
            Err(ret)
        } else {
            Ok(logical_name)
        }
    }
}

/// This function parses the volume group passed in and determines the sizes of
/// the root volume, containerd volume, and csi volume respectively. This should
/// only be used when flashing a new image onto media since storing 256GiB
/// images in GCP is a non starter.
fn get_logical_volume_sizes(volume_group: &LVMVGData) -> (Size, Size, Size) {
    //since this function is being called after the volume group creation I am
    // using the total size instead of free
    let total_size = volume_group.size;
    let _available = total_size - (3 * 256 * BYTE_TO_MEBIBYTE_FACTOR);

    todo!()
}

fn get_partition_path(device: &LoopDevice) -> Option<String> {
    let path = device.path()?;
    let device_string = path.to_str()?;
    Some(format!("{device_string}p2"))
}

pub fn close_lvm() {
    unsafe {
        lvm_rs::bd_lvm_close();
    }
}
