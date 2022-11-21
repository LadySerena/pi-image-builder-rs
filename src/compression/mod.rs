use std::fs::File;
use std::path::Path;
use std::{fs, io};

use bytes::buf::Buf;
use xz2::read::XzDecoder;

pub fn xz_decompress(input_name: String, output_name: String) {
    let input_path = Path::new(input_name.as_str());
    let output_path = Path::new(output_name.as_str());

    if output_path.exists() {
        return;
    }

    let compressed_data = fs::read(input_path).unwrap();
    let mut decompressor = XzDecoder::new(compressed_data.reader());

    let mut output_file = File::create(output_path).unwrap();
    io::copy(&mut decompressor, &mut output_file).unwrap();
}


