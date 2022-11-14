use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::mem::size_of;
use std::ops::Index;
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

pub fn logical_volume_creation(device: &LoopDevice) {
    let init = init_lvm();
    defer!(close_lvm());
    assert!(init, "could not initialize lvm");
    let path = get_partition_path(device).unwrap();
    pv_create(path.as_str()).unwrap();
    vg_create(path.as_str()).unwrap();
    query_volume_groups();
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

fn query_volume_groups() {
    let mut values = Vec::new();
    unsafe {
        let error = ptr::null_mut();
        let output = lvm_rs::bd_lvm_vgs(error);
        let mut index = 0;
        // TODO convert strings to rust strings and maybe find better way of extracting the data
        loop {
            let temp: *mut BDLVMVGdata = *output.add(index);
            if temp.is_null() {
                break;
            }
            values.push(*temp);
            index += 1;
        }
    };
    println!("{}", values[0].size);
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
