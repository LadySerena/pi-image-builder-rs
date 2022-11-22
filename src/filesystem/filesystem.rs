use std::borrow::Borrow;
use std::ffi::CString;
use std::ptr;

use bdfs_sys::{bd_fs_check_deps, bd_fs_close, bd_fs_init, BDExtraArg};
use loopdev::LoopDevice;

use crate::partitioning::ImageInfo;

pub fn create_image_file_systems(image: &ImageInfo) {
    let init = init_fs();
    defer!(close_fs());
    assert!(init, "could not initialize filesystem");

    let boot_path = get_partition_path(image.device.borrow(), "p1").unwrap();
    get_partition_path(image.device.borrow(), "p2");
    create_vfat(boot_path.borrow());
    create_ext4(format!("/dev/{}/{}", image.vg_name, image.lv_name).as_str());
}

fn init_fs() -> bool {
    // TODO re-enable this currently panics because I don't have ntfs stuff and idk
    // why i'd mess with windows let dependency_check = unsafe {
    //     let output = bd_fs_check_deps();
    //     output != 0
    // };
    //
    // if !dependency_check {
    //     return false;
    // }

    unsafe {
        let output = bd_fs_init();
        output != 0
    }
}

fn create_vfat(partition_path: &str) {
    let c_partition_path = CString::new(partition_path).unwrap();
    let c_bit_option = CString::new("-F").unwrap();
    let c_bit_value = CString::new("32").unwrap();
    unsafe {
        let error = ptr::null_mut();
        let args = BDExtraArg {
            opt: c_bit_option.into_raw(),
            val: c_bit_value.into_raw(),
        };
        let arg_ptr: *const BDExtraArg = &args;
        let mut extra_args = [arg_ptr, ptr::null_mut()];
        let extra_args_ptr: *mut *const BDExtraArg = extra_args.as_mut_ptr();
        let success = bdfs_sys::bd_fs_vfat_mkfs(c_partition_path.as_ptr(), extra_args_ptr, error);
        assert_ne!(success, 0);
    }
}

fn create_ext4(partition_path: &str) {
    let c_partition_path = CString::new(partition_path).unwrap();
    unsafe {
        let error = ptr::null_mut();
        let success = bdfs_sys::bd_fs_ext4_mkfs(c_partition_path.as_ptr(), ptr::null_mut(), error);
        assert_ne!(success, 0);
    }
}

fn close_fs() {
    unsafe {
        bd_fs_close();
    }
}

// TODO move this to a util module or something
fn get_partition_path(device: &LoopDevice, partition: &str) -> Option<String> {
    let path = device.path()?;
    let device_string = path.to_str()?;
    Some(format!("{device_string}{partition}"))
}
