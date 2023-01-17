mod fstab;

use self::fstab::create;
use crate::partitioning::ImageInfo;
use alpm::{Alpm, AnyEvent, AnyQuestion, Event, LogLevel, Question, SigLevel, TransFlag};
use indoc::indoc;

use std::fs::{canonicalize, DirBuilder, File, OpenOptions};
use std::io::{Result, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use sys_mount::Mounts;

static MKINITCPIO_CONF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/config/mkinitcpio.conf"
));

static BOOT_TXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/config/boot.txt"));

#[allow(clippy::similar_names)]
pub fn start(info: &dyn ImageInfo, mounts: &Mounts) {
    let root = mounts.0.get(0).unwrap().target_path();
    let boot = mounts.0.get(1).unwrap().target_path();

    copy_embedded_file(root, "/etc/mkinitcpio.conf", MKINITCPIO_CONF);
    copy_embedded_file(boot, "boot.txt", BOOT_TXT);
    create(info, root).unwrap();
    init_keyring(root);
    packages(root);
    create_user("kat", root);
    compile_u_boot_script(root);
}

#[allow(clippy::too_many_lines)]
pub fn packages(root: &Path) {
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

    let pacman_db_path: PathBuf = [root, Path::new("var/lib/pacman")].iter().collect();
    let mut handle = Alpm::new(
        root.as_os_str().as_bytes(),
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

    handle.release().unwrap();
}

fn copy_embedded_file(mount_point: &Path, mounted_filename: &str, content: &str) {
    let config_path = join_paths(mount_point, mounted_filename);
    let mut handle = File::create(config_path).unwrap();
    handle.write_all(content.as_bytes()).unwrap();
}

fn init_keyring(mounted_root: &Path) {
    let key_init_output = Command::new("chroot")
        .arg(mounted_root)
        .arg("pacman-key")
        .arg("--init")
        .output();
    handle_command_output("pacman key init", key_init_output);

    let archlinux_arm_key_add = Command::new("chroot")
        .arg(mounted_root)
        .arg("pacman-key")
        .arg("--populate")
        .arg("archlinuxarm")
        .output();
    handle_command_output("add archlinuxarm key", archlinux_arm_key_add);
}

fn join_paths(initial_path: &Path, to_be_joined: &str) -> PathBuf {
    // this function does not work as expected when there is a leading slash in the `to_be_joined` variable
    let temp = to_be_joined.trim_start_matches('/');
    [initial_path, Path::new(temp)].iter().collect()
}

fn create_user(username: &str, mounted_root: &Path) {
    let sudo_group_name = "katadmin";
    let root = canonicalize(mounted_root).unwrap();

    // SKEL manipulation
    // 78   │ sudo -ukat mkdir -p -m=00700 /home/kat/.ssh
    // 79   │ sudo -ukat touch /home/kat/.ssh/authorized_keys
    // 80   │ chmod 0600 /home/kat/.ssh/authorized_keys
    let ssh_skel = join_paths(mounted_root, "etc/skel/.ssh");
    DirBuilder::new().mode(0o700).create(&ssh_skel).unwrap();
    let key_skel = join_paths(&ssh_skel, "authorized_keys");
    let key_file = File::create(key_skel).unwrap();
    let key_meta = key_file.metadata().unwrap();
    let mut key_permissions = key_meta.permissions();
    key_permissions.set_mode(0o600);

    let create_group = Command::new("groupadd")
        .arg("--root")
        .arg(&root)
        .arg(sudo_group_name)
        .output();

    handle_command_output("create_group", create_group);

    let create_user = Command::new("useradd")
        .arg("--root")
        .arg(&root)
        .arg("--shell")
        .arg("/bin/bash")
        .arg("--create-home")
        .arg("--comment")
        .arg("serena tiede")
        .arg("--group")
        .arg(sudo_group_name)
        .arg(username)
        .output();
    handle_command_output("create user", create_user);

    let delete_alarm = Command::new("userdel")
        .arg("--root")
        .arg(&root)
        .arg("alarm")
        .output();

    handle_command_output("delete alarm", delete_alarm);

    // create sudoers entry
    let sudoers = join_paths(mounted_root, "etc/sudoers");
    let sudo_entry = format!("%{sudo_group_name} ALL=(ALL) NOPASSWD: ALL");
    let mut sudoers_handle = OpenOptions::new().append(true).open(sudoers).unwrap();
    sudoers_handle.write_all(sudo_entry.as_bytes()).unwrap();

    // drop in my ssh key

    let ssh_pub_key = indoc! {"
    ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHRGGe84zs3TxJ8BTbsiVDAsctSf2JF5AS6g/5CyGD2l kat@local-pis
    ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMuS8Kd79MsGzWd68K7WrEIbtBM8WnsqTn0nNz1s+1V7 pi-key-mac    
    "};

    let authorized_keys = join_paths(
        mounted_root,
        format!("home/{username}/.ssh/authorized_keys").as_str(),
    );

    let mut key_handle = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(authorized_keys)
        .unwrap();
    key_handle.write_all(ssh_pub_key.as_bytes()).unwrap();
}

fn compile_u_boot_script(root: &Path) {
    // args taken from https://aur.archlinux.org/cgit/aur.git/tree/mkscr?h=uboot-sunxi

    let boot_txt = join_paths(Path::new("/boot"), "boot.txt");
    let scr = join_paths(Path::new("/boot"), "boot.scr");
    let compile = Command::new("chroot")
        .arg(root)
        .arg("mkimage")
        .arg("--architecture")
        .arg("arm")
        .arg("--os")
        .arg("linux")
        .arg("--type")
        .arg("script")
        .arg("--compression")
        .arg("none")
        .arg("-n")
        .arg("U-Boot boot script")
        .arg("--image")
        .arg(boot_txt)
        .arg(scr)
        .output();
    handle_command_output("compile u-boot", compile);
}

fn handle_command_output(comment: &str, res: Result<Output>) {
    match res {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            println!("{comment} stdout {stdout}");
            eprintln!("{comment} stderr {stderr}");

            assert!(output.status.success());
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::join_paths;

    #[test]
    fn join_pacman() {
        let expected_path = PathBuf::from("./fake-root/fake-dir/fake-file");
        let mounted_root = Path::new("./fake-root");
        let config_path = "fake-dir/fake-file";
        let actual_path = join_paths(mounted_root, config_path);
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn boot_path() {
        let expected_path = PathBuf::from("./fake-root/boot/boot.txt");
        let mounted_boot = Path::new("./fake-root/boot");
        let boot_file = "boot.txt";
        let actual_path = join_paths(mounted_boot, boot_file);
        assert_eq!(actual_path, expected_path);
    }
}
