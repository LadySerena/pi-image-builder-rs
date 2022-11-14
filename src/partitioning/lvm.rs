use std::borrow::Borrow;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, Index};
use std::ptr;
use std::ptr::{slice_from_raw_parts, NonNull};
use std::rc::Rc;
use std::slice::from_raw_parts;

use loopdev::LoopDevice;
use lvm_rs::{gchar, gconstpointer, BDLVMVGdata};

#[derive(Debug)]
struct LvmError {
    message: String,
}

impl Display for LvmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for LvmError {}

#[derive(Clone)]
pub struct LVMVGData {
    pub name: String,
    pub uuid: String,
    pub size: u64,
    pub free: u64,
    pub extent_size: u64,
    pub extent_count: u64,
    pub free_count: u64,
    pub pv_count: u64,
}

pub fn logical_volume_creation(device: &LoopDevice) {
    let init = init_lvm();
    defer!(close_lvm());
    assert!(init, "could not initialize lvm");
    let path = get_partition_path(device).unwrap();
    pv_create(path.as_str()).unwrap();
    let volume_group_name = vg_create(path.as_str()).unwrap();
    let volume_group_data = query_volume_groups(volume_group_name).unwrap();
    let logical_volume_name = lv_create(volume_group_data.borrow()).unwrap();
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

fn pv_create(partition_path: &str) -> Result<(), LvmError> {
    let c_partition = CString::new(partition_path).unwrap();

    unsafe {
        let error = ptr::null_mut();
        let create_output =
            lvm_rs::bd_lvm_pvcreate(c_partition.as_ptr(), 0, 0, ptr::null_mut(), error);
        if create_output == 0 {
            let slice = CStr::from_ptr((*(*error)).message);
            let ret = LvmError {
                message: String::from_utf8_lossy(slice.to_bytes()).to_string(),
            };
            Err(ret)
        } else {
            Ok(())
        }
    }
}

fn vg_create(partition_path: &str) -> Result<&str, LvmError> {
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
            let ret = LvmError {
                message: String::from_utf8_lossy(slice.to_bytes()).to_string(),
            };
            Err(ret)
        } else {
            Ok(volume_group_name)
        }
    }
}

fn query_volume_groups(volume_group_name: &str) -> Option<LVMVGData> {
    let mut values = Vec::new();
    unsafe {
        let error = ptr::null_mut();
        let output = lvm_rs::bd_lvm_vgs(error);
        let mut index = 0;
        loop {
            let temp: *mut BDLVMVGdata = *output.add(index);
            if temp.is_null() {
                break;
            }
            let c_name = CStr::from_ptr((*temp).name);
            let name = String::from_utf8_lossy(c_name.to_bytes()).to_string();
            let c_uuid = CStr::from_ptr((*temp).uuid);
            let uuid = String::from_utf8_lossy(c_uuid.to_bytes()).to_string();
            let size = (*temp).size;
            let free = (*temp).free;
            let extent_size = (*temp).extent_size;
            let extent_count = (*temp).extent_count;
            let free_count = (*temp).free_count;
            let pv_count = (*temp).pv_count;
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
            values.push(safe);
            index += 1;
            temp.drop_in_place();
        }
    };
    values
        .iter()
        .find(|xz2| xz2.name.eq_ignore_ascii_case(volume_group_name))
        .cloned()
}

fn lv_create(volume_group: &LVMVGData) -> Result<String, LvmError> {
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
            let ret = LvmError {
                message: String::from_utf8_lossy(slice.to_bytes()).to_string(),
            };
            Err(ret)
        } else {
            Ok(logical_name.to_string())
        }
    }
}

fn get_partition_path(device: &LoopDevice) -> Option<String> {
    let path = device.path()?;
    let device_string = path.to_str()?;
    Some(format!("{device_string}p2"))
}

fn close_lvm() {
    unsafe {
        lvm_rs::bd_lvm_close();
    }
}
