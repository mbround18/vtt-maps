use blake2::digest::FixedOutput;
use blake2::{Blake2s256, Digest};
use std::path::Path;
use std::{fs, io};

pub fn parse_file_info(path: &str) -> String {
    let mut file = fs::File::open(path).unwrap();
    let mut hasher = Blake2s256::new();
    let n = io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize_fixed();
    format!(
        "Name: {}\nBytes: {}\nHash: {:x}",
        Path::new(path).file_name().unwrap().to_string_lossy(),
        n,
        hash
    )
}

pub fn create_reference_content(reference: String) -> String {
    let handler = "<--- AUTO GENERATED, DO NOT MODIFY --->";
    println!("{}", &reference);
    [handler, reference.as_str(), handler].join("\n")
}
