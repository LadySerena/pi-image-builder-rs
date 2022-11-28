use std::borrow::Borrow;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use alpm::{Alpm, SigLevel};
use sys_mount::Mounts;

pub fn packages(mounts: &Mounts) {
    // https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/transaction.rs

    let mounted_root = mounts.0.get(0).unwrap().target_path();
    let pacman_db_path: PathBuf = [mounted_root, Path::new("var/lib/pacman")].iter().collect();
    let mut handle = Alpm::new(
        mounted_root.as_os_str().as_bytes(),
        pacman_db_path.as_os_str().as_bytes(),
    )
    .unwrap();

    handle
        .register_syncdb_mut("alarm", SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .register_syncdb_mut("community", SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .register_syncdb_mut("core", SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .register_syncdb_mut("extra", SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .add_cachedir(pacman_db_path.as_os_str().as_bytes())
        .unwrap();

    handle.set_check_space(true);

    for db in handle.syncdbs_mut() {
        if let Ok(pkg) = db.pkg("zlib") {
            println!(
                "{} {} {}",
                pkg.name(),
                pkg.desc().unwrap_or("None"),
                pkg.arch().unwrap_or("No Arch")
            );
        }
    }

    // let mut handle = Alpm::new("/", "")
}
