mod files;
mod utils;

use crate::files::dd2vtt::DD2VTTFile;
use crate::utils::{get_files, path_to_thumbnail_path};
use clap::{Arg, Command};
use shared::types::map_reference::MapReference;
use std::path::Path;
use tokio;
use colored::*;

async fn handle_file(file_path: &str) {
    let dd2vtt = DD2VTTFile::from_path(Path::new(file_path).to_path_buf()).await;
    let reference: MapReference = dd2vtt.to_map_reference().await;
    let info_path = Path::new(&file_path).with_extension("info.json");

    // Write info if not found.
    if !info_path.exists() {
        reference.to_file(&info_path);
    }

    // Generate thumbnail if it doesn't exist.
    let thumbnail_path = path_to_thumbnail_path(file_path);
    if !thumbnail_path.exists() {
        println!("\nGenerating thumbnail for: {:?}", file_path);
        dd2vtt.to_thumbnail_file(&thumbnail_path).await;
        print!("{}", "•".green());
        return;
    } else {
        print!("{}", "•".green())
    }

    let info = MapReference::from(&info_path);

    // If old info doesn't match new info, generate new thumbnail & info.
    if info.bytes.ne(&reference.bytes) {
        println!("\nUpdating the following to reference file: {:?}", file_path);
        DD2VTTFile::from_path(Path::new(file_path).to_path_buf())
          .await
          .to_thumbnail_file(&thumbnail_path)
          .await;
        reference.to_file(&info_path);
        print!("{}", "•".green());
    }
}

#[tokio::main]
async fn main() {
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

    let src_dir = std::path::PathBuf::from(input);
    let files = get_files(&src_dir).unwrap();
    let mut handles = vec![];

    for file in files {
        let file_path = file.unwrap().to_str().unwrap().to_string();
        handles.push(tokio::spawn(async move {
            handle_file(&file_path).await;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
    print!("\n");
}
