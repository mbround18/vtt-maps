use glob::glob_with;
use clap::{Arg, App};
// use serde::{Deserialize, Serialize};


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
    let srcdir = std::path::PathBuf::from(matches.value_of("INPUT").unwrap());
    let srcpath = std::fs::canonicalize(&srcdir).unwrap();
    let glob_pattern = std::path::Path::new(&srcpath).join("**").join("*.dd2vtt");
    let files = glob_with(&glob_pattern.to_str().unwrap(), Default::default()).unwrap();

    println!("{:?}", glob_pattern);

    for file in files {
        println!("{}", file.unwrap().to_str().unwrap());
        // let data = std::fs::read_to_string(file.unwrap().as_path()).expect("Unable to read file");
        // let res: DD2VTTFile = serde_json::from_str(&data).expect("Unable to parse");
    }
}
