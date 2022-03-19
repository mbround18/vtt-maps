mod utils;

use crate::utils::{create_reference_content, parse_file_info};
use clap::{Arg, Command};
use glob::{glob_with, Paths, PatternError};
use image::io::Reader as ImageReader;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
struct DD2VTTFile {
    image: String,
}

fn get_files(base_directory: &Path) -> Result<Paths, PatternError> {
    let src_path = std::fs::canonicalize(&base_directory).unwrap();
    let glob_pattern = std::path::Path::new(&src_path).join("**").join("*.dd2vtt");
    glob_with(glob_pattern.to_str().unwrap(), Default::default())
}

fn path_to_thumbnail_path(file_path: &str) -> PathBuf {
    Path::new(file_path)
        .with_extension("")
        .with_extension("preview.png")
}

fn generate_thumbnail(file_path: &str) {
    println!("Generating thumbnail for {}", file_path);
    let data = std::fs::read_to_string(file_path).expect("Unable to read file");
    let res: DD2VTTFile = serde_json::from_str(&data).expect("Unable to parse");
    let bytes = base64::decode(res.image).unwrap();
    let img2 = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    let thumbnail = img2.thumbnail(img2.width() / 16, img2.height() / 16);
    thumbnail.save(path_to_thumbnail_path(file_path)).unwrap();
}

fn write_reference(file_path: &Path, reference: String) {
    if let Err(e) = fs::write(&file_path, &reference) {
        println!("Failed to write to {}", file_path.to_str().unwrap());
        panic!("{}", e)
    };
}

fn handle_file(file_path: &str) {
    println!("Parsing {}", file_path);
    let reference = create_reference_content(parse_file_info(file_path));
    let info_path = Path::new(&file_path).with_extension("info");
    // Write info if not found.
    if !info_path.exists() {
        write_reference(&info_path, reference.clone());
    }

    // Generate thumbnail if it doesnt exist.
    if !path_to_thumbnail_path(file_path).exists() {
        generate_thumbnail(file_path);
        return;
    }

    let info = String::from_utf8(fs::read(&info_path).unwrap()).unwrap();

    // If old info doesnt match new info, generate new thumbnail & info.
    if info.ne(&reference) {
        generate_thumbnail(file_path);
        write_reference(&info_path, reference);
    } else {
        println!("No thumbnail generated for {}", file_path);
    }
}

fn main() {
    let matches = Command::new("thumbnail generator")
        .version("1.0")
        .author("mbround18")
        .about("Parses and minifies the image contained in a dd2vtt file")
        .arg(
            Arg::new("INPUT")
                .help("Sets the input directory to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    println!("Using input file: {}", matches.value_of("INPUT").unwrap());
    let src_dir = std::path::PathBuf::from(matches.value_of("INPUT").unwrap());
    let files = get_files(&src_dir).unwrap();
    for file in files {
        handle_file(file.unwrap().to_str().unwrap())
    }
}
