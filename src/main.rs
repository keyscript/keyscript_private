mod errors;
use std::path::Path;
use std::{env, fs::read_to_string, fs::metadata};
use crate::errors::KeyScriptError;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        KeyScriptError::error(
            KeyScriptError::Error,
            Some("expected file path"),
            None,
            None);
    }
    let file_name = &args[1];
    let path = Path::new(&file_name);
    let main_file_name = path.file_name().unwrap().to_str().unwrap();
    if args.len() == 2 {
        if !file_name.ends_with(".kys") {
            KeyScriptError::error(
                KeyScriptError::Warning,
                Some("file should have a .kys extension!"),
                None,
                None);
        }
        if metadata(file_name).is_err() {
            KeyScriptError::error(
                KeyScriptError::Error,
                Some(&format!("file {main_file_name} does not exist!")),
                None,
                None,
            );
        } else {
            let source = read_to_string(path).expect("failed to read file");
            println!("file's content: {}", source); // start scanning
        }
    }
}
