mod fstab;

use crate::partitioning::ImageInfo;
use alpm::{Alpm, AnyEvent, AnyQuestion, Event, LogLevel, Question, SigLevel, TransFlag};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use sys_mount::Mounts;

pub fn start(info: &dyn ImageInfo, mounts: &Mounts) {
    let mounted_root = mounts.0.get(0).unwrap().target_path();

    fstab::create(info, mounted_root).unwrap();
    init_keyring(mounted_root);
    packages(mounts);
}

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

    handle.set_question_cb((), |question: AnyQuestion, _data| {
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

fn init_keyring(mounted_root: &Path) {
    let pacman_config = get_pacman_paths(mounted_root, "/etc/pacman.conf");
    let pacman_key_dir = get_pacman_paths(mounted_root, "/etc/pacman.d/gnupg");
    let key_init_output = Command::new("pacman-key")
        .arg("--config")
        .arg(pacman_config)
        .arg("--gpgdir")
        .arg(pacman_key_dir)
        .arg("--init")
        .output();
    match key_init_output {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            println!("{stdout}");
            eprintln!("{stderr}");

            assert!(output.status.success());
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}

fn get_pacman_paths(initial_path: &Path, to_be_joined: &str) -> PathBuf {
    // this function does not work as expected when there is a leading slash in the `to_be_joined` variable
    let temp = to_be_joined.trim_start_matches('/');
    [initial_path, Path::new(temp)].iter().collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::get_pacman_paths;

    #[test]
    fn get_pacman_paths_config() {
        let expected_path = PathBuf::from("./fake-root/fake-dir/fake-file");
        let mounted_root = Path::new("./fake-root");
        let config_path = "fake-dir/fake-file";
        let actual_path = get_pacman_paths(mounted_root, config_path);
        assert_eq!(actual_path, expected_path);
    }
}
