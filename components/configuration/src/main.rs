use std::fs::File;

mod configuration;

fn main() {
    // things to configure
    // kernel command line
    // sysctls
    // install packages
    // configure said packages
    // https://github.com/archlinux/alpm.rs/blob/master/alpm/examples/transaction.rs example on installing packages

    let mut f = File::open("lady_tel_test.img").unwrap();
    let mbr = mbrman::
}
