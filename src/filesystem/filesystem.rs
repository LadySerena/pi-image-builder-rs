use bdfs_sys::{bd_fs_check_deps, bd_fs_close, bd_fs_init};
use loopdev::LoopDevice;

pub fn create_image_file_systems(device: &LoopDevice) {
    let init = init_fs();
    defer!(close_fs());
    assert!(init, "could not initialize filesystem");
}

fn init_fs() -> bool {
    let dependency_check = unsafe {
        let output = bd_fs_check_deps();
        output != 0
    };

    if !dependency_check {
        return false;
    }

    unsafe {
        let output = bd_fs_init();
        output != 0
    }
}

fn close_fs() {
    unsafe {
        bd_fs_close();
    }
}
