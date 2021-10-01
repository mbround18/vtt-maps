use glob::{glob_with, Paths, PatternError};
use clap::{Arg, App};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct DD2VTTFile {
    image: String
}


fn get_files(base_directory: &PathBuf) -> Result<Paths, PatternError> {
    let src_path = std::fs::canonicalize(&base_directory).unwrap();
    let glob_pattern = std::path::Path::new(&src_path).join("**").join("*.dd2vtt");
    glob_with(&glob_pattern.to_str().unwrap(), Default::default())
}

fn load_file(file_path: &str) {
    println!("Parsing {}...", &file_path);
    let data = std::fs::read_to_string(file_path).expect("Unable to read file");
    println!("{}",&data);
    let res: DD2VTTFile = serde_json::from_str(&data).expect("Unable to parse");
    println!("{}", serde_json::to_string(&res).unwrap())
}


fn main() {
    let matches = App::new("dd2vtt parser")
        .version("1.0")
        .author("mbround18")
        .about("Parses and minifies the image contained in a dd2vtt file")
        .arg(Arg::new("INPUT")
            .about("Sets the input directory to use")
            .required(true)
            .index(1))
        .get_matches();


    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    println!("Using input file: {}", matches.value_of("INPUT").unwrap());
    let src_dir = std::path::PathBuf::from(matches.value_of("INPUT").unwrap());
    let files = get_files(&src_dir).unwrap();

    for file in files {
        load_file(file.unwrap().to_str().unwrap())
        // let data = std::fs::read_to_string(file.unwrap().as_path()).expect("Unable to read file");
        // let res: DD2VTTFile = serde_json::from_str(&data).expect("Unable to parse");
    }
}
