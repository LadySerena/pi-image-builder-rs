use std::ffi::c_void;
use std::ptr;

use glib::ffi::gboolean;
use glib::Error;

#[link(name = "bd_lvm")]
extern "C" {
    fn bd_lvm_check_deps() -> gboolean;
    fn bd_lvm_init() -> gboolean;
    fn bd_lvm_pvs(error: *mut *mut Error);
    fn bd_lvm_close();
}

pub fn testing() {
    let meow = unsafe { bd_lvm_check_deps() };
    assert_ne!(meow, 0);
    let init = unsafe { bd_lvm_init() };
    assert_ne!(init, 0);
    unsafe {
        let mut error = ptr::null_mut();
        bd_lvm_pvs(&mut error);
    }

    unsafe {
        bd_lvm_close();
    }
}
