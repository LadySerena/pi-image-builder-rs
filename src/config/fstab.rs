use std::ffi::OsStr;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use askama::{Error, Template};

use crate::partitioning::ImageInfo;

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

pub fn create(info: &dyn ImageInfo, mounted_root: &Path) -> Result<(), Error> {
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

    let fstab_dir: PathBuf = [mounted_root.as_os_str(), OsStr::new("etc")]
        .iter()
        .collect();

    create_dir_all(fstab_dir.as_path()).unwrap();

    let fstab: PathBuf = [fstab_dir.as_os_str(), OsStr::new("fstab")]
        .iter()
        .collect();

    let mut file = File::create(fstab).unwrap();

    file.write_all(data.as_bytes()).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {

    use askama::Error;
    use indoc::indoc;

    use tempdir::TempDir;

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
        let dir = TempDir::new("mock_root_mount").unwrap();
        create(&mock_info, dir.path())?;
        dir.close().unwrap();
        Ok(())
    }
}
