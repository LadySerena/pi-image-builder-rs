fn main() {
    println!("cargo:rustc-link-lib=bd_lvm");
    println!("cargo:rustc-link-lib=bd_fs");
}
