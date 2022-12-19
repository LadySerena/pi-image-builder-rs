mod fstab;

use std::borrow::{Borrow, BorrowMut};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use alpm::{
    Alpm, AnyEvent, AnyQuestion, Db, DbMut, Event, EventType, LogLevel, Package, Pkg, Question,
    SigLevel, TransFlag,
};
use sys_mount::Mounts;

#[allow(clippy::too_many_lines)]
pub fn packages(mounts: &Mounts) {
    const ALARM_REPO_NAME: &str = "alarm";
    const COMMUNITY_REPO_NAME: &str = "community";
    const CORE_REPO_NAME: &str = "core";
    const EXTRA_REPO_NAME: &str = "extra";

    // https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/transaction.rs

    let core_packages = vec![
        "openssh",
        "ca-certificates",
        "curl",
        "gnupg",
        "sudo",
        "perl",
        "lvm2",
        "bash",
        "util-linux",
        "grep",
        "lsb-release",
        "open-iscsi",
        "containerd",
        "mkinitcpio",
        "uboot-tools",
        "cni-plugins",
        "crictl",
        "kubeadm",
        "kubectl",
        "kubelet",
        "wget",
        "lm_sensors",
        "htop",
        "nftables",
        "conntrack-tools",
    ];

    let mounted_root = mounts.0.get(0).unwrap().target_path();
    let pacman_db_path: PathBuf = [mounted_root, Path::new("var/lib/pacman")].iter().collect();
    let mut handle = Alpm::new(
        mounted_root.as_os_str().as_bytes(),
        pacman_db_path.as_os_str().as_bytes(),
    )
    .unwrap();

    handle.set_log_cb((), |loglevel: LogLevel, msg, _data| {
        if !loglevel.eq(&LogLevel::DEBUG) && !loglevel.eq(&LogLevel::FUNCTION) {
            print!("{loglevel:?} {msg}");
        }
    });

    handle.set_event_cb((), |event: AnyEvent, _data| match event.event() {
        Event::TransactionStart => println!("transaction start"),
        Event::TransactionDone => println!("transaction done"),
        Event::RetrieveFailed => println!("package retrieval failed"),
        _ => (),
    });

    handle.set_question_cb((), |question: AnyQuestion, data| {
        if let Question::Conflict(mut question) = question.question() {
            question.set_remove(true);
            println!(
                "package1: {} vs package2: {}",
                question.conflict().package1(),
                question.conflict().package2()
            );
        }
    });

    handle
        .register_syncdb_mut(ALARM_REPO_NAME, SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .register_syncdb_mut(COMMUNITY_REPO_NAME, SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .register_syncdb_mut(CORE_REPO_NAME, SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .register_syncdb_mut(EXTRA_REPO_NAME, SigLevel::USE_DEFAULT)
        .unwrap();

    handle
        .add_cachedir(pacman_db_path.as_os_str().as_bytes())
        .unwrap();

    handle.set_check_space(true);

    let alarm = handle
        .syncdbs_mut()
        .iter()
        .find(|db| db.name() == ALARM_REPO_NAME)
        .unwrap();
    alarm
        .add_server("http://mirror.archlinuxarm.org/aarch64/alarm")
        .unwrap();

    let community = handle
        .syncdbs_mut()
        .iter()
        .find(|db| db.name() == COMMUNITY_REPO_NAME)
        .unwrap();
    community
        .add_server("http://mirror.archlinuxarm.org/aarch64/community")
        .unwrap();

    let core = handle
        .syncdbs_mut()
        .iter()
        .find(|db| db.name() == CORE_REPO_NAME)
        .unwrap();
    core.add_server("http://mirror.archlinuxarm.org/aarch64/core")
        .unwrap();

    let extra = handle
        .syncdbs_mut()
        .iter()
        .find(|db| db.name() == EXTRA_REPO_NAME)
        .unwrap();
    extra
        .add_server("http://mirror.archlinuxarm.org/aarch64/extra")
        .unwrap();

    handle.syncdbs_mut().update(false).unwrap();

    let flags = TransFlag::NEEDED | TransFlag::RECURSE;

    handle.trans_init(flags).unwrap();

    let mut pkgs = Vec::new();

    for core_package in core_packages.as_slice() {
        for db in handle.syncdbs() {
            if let Ok(pkg) = db.pkg(*core_package) {
                pkgs.push(pkg);
            }
        }
    }

    assert_eq!(pkgs.len(), core_packages.len());

    for x in pkgs {
        handle.trans_add_pkg(x).unwrap();
    }

    handle.sync_sysupgrade(false).unwrap();

    handle.trans_prepare().unwrap();

    handle.trans_commit().unwrap();

    handle.trans_release().unwrap();
}
