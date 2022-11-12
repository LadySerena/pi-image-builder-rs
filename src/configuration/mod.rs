// - kernel stuff
// - copy config files from embedded fs
// - install packages
// - install k8s related things
// - configure cloud init
// - write up fstab entries

use crate::configuration::models::SysctlList;

mod kernel_config;
mod models;

pub fn configure_image() {
    // TODO have a sysctl builder
    // Pass in embedded with file path key
    // package list and trait
    // k8s install via raw binaries
    // models for cloud init
    // fstab module

    // kernel_config::hello_world("10-kubernetes.conf").unwrap();
}
