use colored::*;
#[derive(Debug, PartialEq)]
pub enum KeyScriptError {
    ScannerError,
    ParserError,
    RuntimeError,
    Error,
    Warning,
}

impl KeyScriptError {
    pub fn error(error_type: KeyScriptError, msg: Option<&str>, line: Option<usize>, filename: Option<&str>) {
        if error_type != Self::Warning {
            print!("{} ",format!("[{error_type:?}]").red());
        }
        else {
            print!("{} ",format!("[{error_type:?}]").yellow());
        }
        if msg.is_some() {
            print!("{} ", msg.unwrap().red());
        } else {
            print!("{} ", "unknown error".red());
        }
        if line.is_some() {
            print!("{} ", format!("at line {}", line.unwrap()).red());
        }
        if filename.is_some() {
            println!("{}", format!("in file {}", filename.unwrap()).red());
        } else {
            println!();
        }
        match error_type {
            Self::ScannerError => (),
            KeyScriptError::Warning => (),
            _ => std::process::exit(0),
        }
    }
}
