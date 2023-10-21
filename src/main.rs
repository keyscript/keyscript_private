mod errors;
mod scanner;
use scanner::Scanner;
mod parser;
mod compiler;
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
            let mut scanner = scanner::Scanner::new(&source, main_file_name);
            let tokens = scanner.scan_tokens();
            for token in tokens {
                println!("{:?}", token);
            }
        }
    }
}
