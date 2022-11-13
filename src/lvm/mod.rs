use std::ffi::CStr;
use std::ptr;

use glib::ffi::gboolean;
use glib::Error;

pub fn testing() {
    let meow = unsafe { lvm_rs::bd_lvm_check_deps() };
    assert_ne!(meow, 0);
    let init = unsafe { lvm_rs::bd_lvm_init() };
    assert_ne!(init, 0);
    unsafe {
        // ok this totally did work tho and now it's broken?
        let mut error = ptr::null_mut();
        let pv_data = lvm_rs::bd_lvm_pvs(&mut error);
        assert_eq!(error, ptr::null_mut());
    }

    unsafe {
        lvm_rs::bd_lvm_close();
    }
}
