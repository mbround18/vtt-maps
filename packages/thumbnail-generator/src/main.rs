mod files;
mod utils;

use crate::files::dd2vtt::DD2VTTFile;
use crate::utils::{get_files, path_to_thumbnail_path};
use clap::{Arg, Command};
use shared::types::map_reference::MapReference;
use std::path::Path;

fn handle_file(file_path: &str) {
    println!("Parsing {}", file_path);
    let dd2vtt = DD2VTTFile::from(String::from(file_path));
    let reference: MapReference = dd2vtt.clone().into();
    let info_path = Path::new(&file_path).with_extension("info.json");

    // Write info if not found.
    if !info_path.exists() {
        reference.to_file(&info_path);
    }

    // Generate thumbnail if it doesnt exist.
    let thumbnail_path = path_to_thumbnail_path(file_path);
    if !thumbnail_path.exists() {
        dd2vtt.to_thumbnail_file(thumbnail_path);
        return;
    }

    let info = MapReference::from(&info_path);

    // If old info doesnt match new info, generate new thumbnail & info.
    if info.bytes.ne(&reference.bytes) {
        DD2VTTFile::from(String::from(file_path)).to_thumbnail_file(thumbnail_path);
        reference.to_file(&info_path);
    } else {
        println!("No thumbnail generated for {}", file_path);
    }
}

fn main() {
    let matches = Command::new("dd2vtt parser")
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
    let input: &String = matches.get_one("INPUT").unwrap();
    println!("Using input file: {}", input);

    // .value_of("INPUT").unwrap());
    let src_dir = std::path::PathBuf::from(input);
    let files = get_files(&src_dir).unwrap();
    for file in files {
        handle_file(file.unwrap().to_str().unwrap())
    }
}
