use std::path::PathBuf;

use askama::Template;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_happy() {
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
        // TODO make this test actually test
        print!("{}", temp.render().unwrap());
    }
}
