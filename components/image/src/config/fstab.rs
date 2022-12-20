use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use askama::{Error, Template};
use sys_mount::Mounts;

use crate::partitioning::{ImageInfo, RuntimeImageInfo};

#[derive(Template)]
#[template(path = "etc/fstab")]
struct FStabTemplate {
    boot_label: String,
    volumes: Vec<VolumeEntry>,
}

struct VolumeEntry {
    mapper: String,
    mount_point: String,
    filesystem: String,
}

pub fn create_fstab(info: &dyn ImageInfo, mounted_root: &Path) -> Result<(), Error> {
    // in mount/mod.rs the order of mounts are root, boot, ...., other junk

    let volumes = vec![VolumeEntry {
        mapper: info.root_path().to_str().unwrap().to_string(),
        mount_point: "/".to_string(),
        filesystem: "ext4".to_string(),
    }];

    let template = FStabTemplate {
        boot_label: "BOOT".to_string(),
        volumes,
    };

    let data = template.render()?;

    let fstab: PathBuf = [mounted_root.as_os_str(), OsStr::new("etc/fstab")]
        .iter()
        .collect();

    let mut file = File::create(fstab).unwrap();

    file.write_all(data.as_bytes()).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use askama::Error;
    use indoc::indoc;
    use loopdev::LoopDevice;

    use crate::partitioning::MockImageInfo;

    use super::*;

    #[test]
    fn create_fstab_happy() -> Result<(), Error> {
        let mut mock_info: MockImageInfo = MockImageInfo::new();
        let root = PathBuf::from(r"/dev/mapper/rootvg-rootlv");

        mock_info
            .expect_root_path()
            .times(1)
            .returning(move || root.clone());
        let mut dir = env::temp_dir();
        dir.push("mock_root_mount");

        // TODO make test not panic
        // called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound,
        // message: "No such file or directory" }
        // thread 'config::fstab::tests::create_fstab_happy' panicked at 'called
        // `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message:
        // "No such file or directory" }', components/image/src/config/fstab.rs:44:40
        // stack backtrace:
        //    0: rust_begin_unwind
        //              at
        // /rustc/0468a00ae3fd6ef1a6a0f9eaf637d7aa9e604acc/library/std/src/panicking.rs:
        // 575:5    1: core::panicking::panic_fmt
        //              at
        // /rustc/0468a00ae3fd6ef1a6a0f9eaf637d7aa9e604acc/library/core/src/panicking.
        // rs:64:14    2: core::result::unwrap_failed
        //              at
        // /rustc/0468a00ae3fd6ef1a6a0f9eaf637d7aa9e604acc/library/core/src/result.rs:
        // 1790:5    3: core::result::Result<T,E>::unwrap
        //              at
        // /rustc/0468a00ae3fd6ef1a6a0f9eaf637d7aa9e604acc/library/core/src/result.rs:
        // 1112:23    4: pi_image_builder_rs::config::fstab::create_fstab
        //              at ./src/config/fstab.rs:44:20
        //    5: pi_image_builder_rs::config::fstab::tests::create_fstab_happy
        //              at ./src/config/fstab.rs:75:9
        //    6: pi_image_builder_rs::config::fstab::tests::create_fstab_happy::{{closure}}
        //              at ./src/config/fstab.rs:64:32
        //    7: core::ops::function::FnOnce::call_once
        //              at
        // /rustc/0468a00ae3fd6ef1a6a0f9eaf637d7aa9e604acc/library/core/src/ops/
        // function.rs:507:5    8: core::ops::function::FnOnce::call_once
        //              at
        // /rustc/0468a00ae3fd6ef1a6a0f9eaf637d7aa9e604acc/library/core/src/ops/
        // function.rs:507:5 note: Some details are omitted, run with
        // `RUST_BACKTRACE=full` for a verbose backtrace.
        //
        //
        // Process finished with exit code 101
        create_fstab(&mock_info, dir.as_path()).expect("TODO: panic message");

        Ok(())
    }

    #[test]
    fn render_happy() -> Result<(), Error> {
        let expected = indoc! {"
            # Static information about the filesystems.
            # See fstab(5) for details.
            
            # <file system> <dir> <type> <options> <dump> <pass>
            
            # boot filesystem
            LABEL=BOOT /boot vfat defaults 0 0
            # other filesystems
            # TODO sort out filesystem mount options
            
            /dev/mapper/rootvg-rootlv / ext4 defaults 0 1
            
            /dev/mapper/rootvg-csilv /var/lib/longhorn ext4 defaults 0 1
        "};
        let volumes = vec![
            VolumeEntry {
                mapper: "/dev/mapper/rootvg-rootlv".to_string(),
                mount_point: "/".to_string(),
                filesystem: "ext4".to_string(),
            },
            VolumeEntry {
                mapper: "/dev/mapper/rootvg-csilv".to_string(),
                mount_point: "/var/lib/longhorn".to_string(),
                filesystem: "ext4".to_string(),
            },
        ];

        let temp = FStabTemplate {
            boot_label: "BOOT".to_string(),
            volumes,
        };

        let render = temp.render()?;
        assert_eq!(expected, render.as_str());
        Ok(())
    }
}
