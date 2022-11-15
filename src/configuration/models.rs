use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::path::PathBuf;
use std::{fs, io};

use crate::configuration;

#[derive(Debug)]
pub enum SysctlErr {
    DropInError(SysctlDropInError),
    IoError(io::Error),
}

#[derive(Debug)]
pub struct SysctlDropInError {
    path: PathBuf,
}

impl Error for SysctlDropInError {}

impl Display for SysctlDropInError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is not a valid sysctl drop in directory, the path must be a directory and exist",
            self.path.display()
        )
    }
}

impl SysctlDropInError {
    fn new(path: PathBuf) -> Self {
        SysctlDropInError { path }
    }
}

pub struct SysctlList {
    sysctls: Vec<SysctlEntry>,
    drop_in_file: PathBuf,
}

impl SysctlList {
    pub fn new(sysctls: &[SysctlEntry], file_name: &str) -> Result<Self, SysctlDropInError> {
        let drop_in = Box::new(PathBuf::from("/etc/sysctl.d/"));

        if is_valid_sysctl_dir(&drop_in) {
            Ok(SysctlList {
                sysctls: sysctls.to_vec(),
                drop_in_file: drop_in.join(file_name),
            })
        } else {
            Err(SysctlDropInError::new(drop_in.to_path_buf()))
        }
    }

    pub fn collect_sysctls(&self) -> String {
        self.sysctls
            .iter()
            .map(configuration::models::SysctlEntry::write_to_string)
            .collect::<Vec<String>>()
            .join("\n")
            .add("\n")
    }

    pub fn write_all_to_file(&self) -> io::Result<()> {
        let data = self.collect_sysctls();
        fs::write(self.drop_in_file.clone(), data)
    }
}

#[derive(Clone)]
pub struct SysctlEntry {
    key: String,
    value: String,
}

impl SysctlEntry {
    pub fn new(key: &str, value: &str) -> SysctlEntry {
        SysctlEntry {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn write_to_string(&self) -> String {
        format!("{}={}", self.key, self.value)
    }
}

fn is_valid_sysctl_dir(drop_in: &PathBuf) -> bool {
    let is_dir = drop_in.is_dir();
    let exists = drop_in.exists();
    is_dir & exists
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::configuration::models::{SysctlEntry, SysctlList};

    #[test]
    fn validate_sysctl_write() {
        let expected = indoc! {"
            net.bridge.bridge-nf-call-ip6tables=1
            net.bridge.bridge-nf-call-iptables=1
        "};

        let tested = SysctlList::new(
            &[
                SysctlEntry::new("net.bridge.bridge-nf-call-ip6tables", "1"),
                SysctlEntry::new("net.bridge.bridge-nf-call-iptables", "1"),
            ],
            "meow.conf",
        )
        .unwrap()
        .collect_sysctls();

        assert_eq!(tested, expected)
    }
}
