use std::fs::File;
use std::path::Path;
use std::{fs, io};

use bytes::buf::*;
use xz2::read::XzDecoder;

pub fn xz_decompress(file_name: String, output_name: String) {
    let path = Path::new(file_name.as_str());
    let compressed_data = fs::read(path).unwrap();
    let mut decompressor = XzDecoder::new(compressed_data.reader());
    let mut output_file = File::create(Path::new(output_name.as_str())).unwrap();
    io::copy(&mut decompressor, &mut output_file).unwrap();
}
