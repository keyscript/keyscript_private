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
        error_type.print();
        if error_type != Self::Warning {
            print!("{} ",format!("[{}]", error_type.print()).red());
        }
        else {
            print!("{} ",format!("[{}]", error_type.print()).yellow());
        }
        if msg.is_some() {
            print!("{} ", msg.unwrap().red());
        } else {
            print!("{} ", "unknown error".red());
        }
        if line.is_some() {
            print!("{} ", format!("{} line {}", "at".red(), line.unwrap()).blue());
        }
        if filename.is_some() {
            println!("{}", format!("{} {}", "in file".red(), filename.unwrap()).blue());
        } else {
            println!();
        }
        match error_type {
            Self::ScannerError => (),
            Self::Warning => (),
            _ => std::process::exit(0),
        }
    }

    fn print(&self) -> String {
        match self {
            KeyScriptError::ScannerError => String::from("SCANNER ERROR"),
            KeyScriptError::ParserError => String::from("PARSER ERROR"),
            KeyScriptError::RuntimeError => String::from("RUNTIME ERROR"),
            KeyScriptError::Error => String::from("ERROR"),
            KeyScriptError::Warning => String::from("WARNING"),
        }
    }
}
