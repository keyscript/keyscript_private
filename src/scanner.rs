use crate::errors::KeyScriptError;
use std::iter::Peekable;
use std::str::Chars;

pub struct Scanner<'a> {
    pub source: &'a str,
    pub chars: Peekable<Chars<'a>>,
    pub line: usize,
    pub tokens: Vec<Token>,
    filename: &'a str,
    had_error: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, filename: &'a str) -> Scanner<'a> {
        Scanner {
            source,
            chars: source.chars().peekable(),
            line: 1,
            tokens: Vec::new(),
            filename,
            had_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while let Some(ch) = self.chars.next() {
            match ch {
                '(' => self.make_token(TokenType::LeftParen, None),
                ')' => self.make_token(TokenType::RightParen, None),
                '{' => self.make_token(TokenType::LeftBrace, None),
                '}' => self.make_token(TokenType::RightBrace, None),
                ',' => self.make_token(TokenType::Comma, None),
                '.' => self.make_token(TokenType::Dot, None),
                '-' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::MinusEqual, None);
                    } else {
                        self.make_token(TokenType::Minus, None);
                    }
                }
                '+' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::PlusEqual, None);
                    } else {
                        self.make_token(TokenType::Plus, None);
                    }
                }
                '*' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::StarEqual, None);
                    } else {
                        self.make_token(TokenType::Star, None);
                    }
                }
                '/' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::SlashEqual, None);
                    } else if let Some(&'/') = self.chars.peek() {
                        while let Some(&ch) = self.chars.peek() {
                            if ch == '\n' {
                                break;
                            }
                            self.chars.next();
                        }
                    } else {
                        self.make_token(TokenType::Slash, None);
                    }
                }
                '%' => self.make_token(TokenType::Modulo, None),
                ';' => self.make_token(TokenType::Semicolon, None),
                '[' => self.make_token(TokenType::LeftSquare, None),
                ']' => self.make_token(TokenType::RightSquare, None),
                '!' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::BangEqual, None);
                    } else {
                        self.make_token(TokenType::Bang, None);
                    }
                }
                '=' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::EqualEqual, None);
                    } else {
                        self.make_token(TokenType::Equal, None);
                    }
                }
                '<' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::LessEqual, None);
                    } else {
                        self.make_token(TokenType::Less, None);
                    }
                }
                '>' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::GreaterEqual, None);
                    } else {
                        self.make_token(TokenType::Greater, None);
                    }
                }
                '&' => {
                    if let Some(&'&') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::And, None);
                    } else {
                        self.error("expected &");
                    }
                }
                '|' => {
                    if let Some(&'|') = self.chars.peek() {
                        self.chars.next();
                        self.make_token(TokenType::Or, None);
                    } else {
                        self.error("expected |");
                    }
                }
                '"' => self.string(),
                ' ' => (),
                '\r' => (),
                '\t' => (),
                '\n' => self.line += 1,
                _ => {
                    if ch.is_ascii_digit() {
                        self.number(ch);
                    } else if ch.is_ascii_alphabetic() {
                        self.identifier(ch);
                    } else {
                        self.error("unexpected character");
                        self.had_error = true;
                    }
                }
            }
        }
        self.make_token(TokenType::Eof, None);
        if self.had_error {
            std::process::exit(0); //scanner had an error, errors are printed and the program exits
        } else {
            std::mem::take(&mut self.tokens) //scanner had no errors, tokens are taken and the program continues
        }
    }

    fn string(&mut self) {
        let mut string = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch == '"' {
                self.chars.next();
                self.make_token(TokenType::Value, Some(Value::String(string)));
                return;
            }
            string.push(ch);
            self.chars.next();
        }
        self.error("unterminated string");
    }

    fn number(&mut self, first: char) {
        let mut number = String::from(first);
        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        if let Some(&'.') = self.chars.peek() {
            number.push('.');
            self.chars.next();
            while let Some(&ch) = self.chars.peek() {
                if ch.is_ascii_digit() {
                    number.push(ch);
                    self.chars.next();
                } else {
                    break;
                }
            }
            self.make_token(TokenType::Value, Some(Value::Float(number.parse::<f64>().unwrap())));
        } else {
            self.make_token(TokenType::Value, Some(Value::Int(number.parse::<i64>().unwrap())));
        }
    }

    fn identifier(&mut self, first: char) {
        let mut identifier = String::from(first);
        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_alphanumeric() {
                identifier.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        match identifier.as_str() {
            "if" => self.make_token(TokenType::If, None),
            "else" => self.make_token(TokenType::Else, None),
            "while" => self.make_token(TokenType::While, None),
            "print" => self.make_token(TokenType::Print, None),
            "return" => self.make_token(TokenType::Return, None),
            "true" => self.make_token(TokenType::Value, Some(Value::Bool(true))),
            "false" => self.make_token(TokenType::Value, Some(Value::Bool(false))),
            "int" => self.make_token(TokenType::Int, None),
            "float" => self.make_token(TokenType::Float, None),
            "string" => self.make_token(TokenType::String, None),
            "bool" => self.make_token(TokenType::Bool, None),
            "pub" => self.make_token(TokenType::Pub, None),
            _ => self.make_token(TokenType::Identifier, Some(Value::String(identifier))),
        }
    }

    fn make_token(&mut self, tt: TokenType, literal: Option<Value>) {
        self.tokens.push(Token {
            tt,
            literal,
            line: self.line,
        });
    }

    fn error(&mut self, msg: &str) {
        self.had_error = true;
        KeyScriptError::error(
            KeyScriptError::ScannerError,
            Some(msg),
            Some(self.line),
            Some(self.filename),
        );
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Float(f64),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub struct Token{
    pub tt: TokenType,
    pub literal: Option<Value>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    MinusEqual,
    Plus,
    PlusEqual,
    Slash,
    SlashEqual,
    Star,
    StarEqual,
    Modulo,
    Semicolon,
    LeftSquare,
    RightSquare,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,

    Identifier,
    String,
    Int,
    Float,
    Bool,
    Value,
    If,
    Else,
    While,
    Pub,
    Print,
    Return,
    Eof,
}