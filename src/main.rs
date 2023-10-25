mod errors;
mod scanner;
mod parser;
mod compiler;
mod ast;
use std::path::Path;
use std::{env, fs::read_to_string, fs::metadata};
use std::io::Write;
use crate::errors::KeyScriptError;

fn main() {
    // std::env::set_var("RUST_BACKTRACE", "5");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        KeyScriptError::error(
            KeyScriptError::Error,
            Some("expected file path"),
            None,
            None);
    }
    let file_name;
    let mut is_wat = false;
    if args.len() > 2 && args [2] == "debug" {
        is_wat = true;
    }
    if &args[1] == "init" {
        let loop_code = r#"int fib(int n) {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}"#;
        let mut file = std::fs::File::create("index.kys").unwrap();
        file.write_all(loop_code.as_bytes()).expect("Failed to write to file");
        let html_code = r#"<!DOCTYPE html>
<html>
<head>
    <title>WebAssembly Test</title>
    <style>
        body {
            background-color: black;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
        }

        #output {
            color: white;
            font-size: 24px;
            text-align: center;
        }

        #error {
            color: red;
            font-size: 24px;
            text-align: center;
        }
    </style>
</head>
<body>
<div id="output"></div>
<div id="error"></div>
<script>
    let imports = {
        wasm: {
            memory: new WebAssembly.Memory({initial: 256}), // 1 page = 64KB, 256 pages = much storage
        },
        console: {
            log: function (offset, length) {
                console.log(new TextDecoder('utf8').decode(new Uint8Array(imports.wasm.memory.buffer, offset, length)));
            }
        }
    };
    fetch('index.wasm') // file name!!
        .then(response => response.arrayBuffer())
        .then(bytes => {
            return WebAssembly.instantiate(bytes, imports)
        })
        .then(result => {
            const returnValue = result.instance.exports.fib(BigInt(40)); // use BigInt for ints, use exports.<function name> for functions.
            if (returnValue) {
                document.getElementById('output').textContent = `Function returned: ${returnValue}`;
            } else {
                document.getElementById('output').textContent = `No output, check the console`;
            }

        })
        .catch(error => {
            document.getElementById('error').textContent = `Error loading WebAssembly: ${error.message}`;
        })
</script>
</body>
</html>
"#;

        let mut file = std::fs::File::create("index.html").unwrap();
        file.write_all(html_code.as_bytes()).expect("Failed to write HTML code to file");
        file_name = "index.kys";
    } else {
        file_name = &args[1];
    }
    let path = Path::new(&file_name);
    let main_file_name = path.file_name().unwrap().to_str().unwrap();
    if args.len() >= 2 {
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
            // for i in &tokens {
            //     println!("{:?}", i);
            // }
            let mut parser = parser::Parser::new(tokens, main_file_name);
            let mut comp = compiler::Compiler::new(parser.parse(), parser.vars, file_name);
            // println!("{:?}", parser.parse());
            comp.compile(is_wat);

        }
    }
}

//todo list:
//polish the error messages
//debug
//break keyword
//release!
//arrays
//try to add keyscript compile to npm run
//start implementing features