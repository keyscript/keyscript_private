use std::collections::HashMap;
use wasm_encoder::{BlockType, CodeSection, ConstExpr, DataSection, DataSegment, EntityType, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, MemorySection, MemoryType, Module, TypeSection, ValType};
use std::fs;
use colored::Colorize;
use crate::{ast::Expr, scanner::{Value, TokenType}};
use crate::ast::Stmt;
use crate::errors::KeyScriptError;
use wasmparser::Parser;

//REMINDER of wasm structure:
//first types
//then functions and indexes
//then exports
//then code
//booleans are stored in i32!!
//Strings are stored in i32 as well!! (offset)

pub struct Compiler {
    module: Module,
    vars: HashMap<String, (u32, TokenType)>,
    vars_count: u32,
    funcs: HashMap<String, (u32, TokenType)>,
    vars1: Vec<TokenType>,
    code: Vec<Stmt>,
    path: String,
    strings: Vec<u8>,
    offsets: HashMap<i32, i32>, //offset, length
    string_vars: HashMap<String, i32>, //name, offset
    kys_funcs: Vec<Stmt>,
}

impl Compiler {
    pub fn new(code: Vec<Stmt>, vars1: Vec<TokenType>, filename: &str) -> Compiler {
        Compiler {
            module: Module::new(),
            vars: HashMap::new(),
            vars_count: 0,
            funcs: HashMap::new(),
            vars1,
            code,
            path: filename.to_string().replace(".kys", ".wasm"),
            strings: Vec::new(),
            offsets: HashMap::new(),
            string_vars: HashMap::new(),
            kys_funcs: Vec::new()
        }
    }

    pub fn compile(&mut self, is_wat: bool) {
        while let Some(stmt) = self.code.first() {
            match stmt {
                Stmt::Fn {..} => {
                    self.kys_funcs.push(self.code.remove(0));
                }
                _ => break,
            }
        }
        let counter = 2;
        for i in self.kys_funcs.iter() {
            match i {
                Stmt::Fn {
                    name,
                    return_type,
                    ..
                } => {
                    self.funcs.insert(name.clone(), (counter, return_type.clone()));
                }
                _ => self.error("unreachable?"),
            }
        }

        let mut types = TypeSection::new();
        let params = vec![ValType::I32, ValType::I32];
        let results = vec![];
        types.function(params, results); //print, 0
        let params = vec![];
        let results = vec![];
        types.function(params, results); //main function, 1
        for i in self.kys_funcs.iter() {
            if let Stmt::Fn {
                params,
                return_type,
                ..
            } = i {
                let mut params1 = vec![];
                for param in params {
                    self.strings.extend_from_slice(param.1.literal.clone().unwrap().as_str().as_bytes());
                    self.offsets.insert(self.strings.len() as i32 - param.1.literal.clone().unwrap().as_str().len() as i32, param.1.literal.clone().unwrap().as_str().len() as i32);
                    self.string_vars.insert(param.1.literal.clone().unwrap().as_str().to_string(), self.strings.len() as i32 - param.1.literal.clone().unwrap().as_str().len() as i32);
                    params1.push(match param.0 {
                        TokenType::Int => ValType::I64,
                        TokenType::Float => ValType::F64,
                        TokenType::Bool => ValType::I32,
                        TokenType::String => ValType::I32,
                        _ => panic!("unreachable?"),
                    });
                }
                let mut results1 = vec![];
                match return_type {
                    TokenType::Int => results1.push(ValType::I64),
                    TokenType::Float => results1.push(ValType::F64),
                    TokenType::Bool => results1.push(ValType::I32),
                    TokenType::String => results1.push(ValType::I32),
                    TokenType::Void => {},
                    _ => panic!("unreachable?"),
                }
                types.function(params1, results1);
            }
        }
        self.module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("wasm", "memory", EntityType::Memory(MemoryType{
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
        }));
        imports.import("console", "log", EntityType::Function(0));
        self.module.section(&imports);

        let mut functions = FunctionSection::new();
        let type_index = 1;
        functions.function(type_index);
        let mut counter = 2;
        for _ in self.kys_funcs.iter() {
            functions.function(counter);
            counter += 1;
        }
        self.module.section(&functions);

        // let mut memory = MemorySection::new();
        // memory.memory(MemoryType {
        //     minimum: 1,
        //     maximum: None,
        //     memory64: false,
        //     shared: false,
        // });
        // self.module.section(&memory);

        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 1);
        counter = 2;
        for i in self.kys_funcs.iter() {
            if let Stmt::Fn {
                name,
                ..
            } = i {
                exports.export(name.as_str(), ExportKind::Func, counter);
                counter += 1;
            }
        }
        self.module.section(&exports);

        let mut codes = CodeSection::new();
        let mut locals = vec![];
        for var in &self.vars1 {
            match var {
                TokenType::Int => locals.push((1,ValType::I64)),
                TokenType::Float => locals.push((1,ValType::F64)),
                TokenType::Bool => locals.push((1,ValType::I32)),
                TokenType::String => locals.push((1,ValType::I32)),
                _ => self.error("unreachable?"),
            }
        }
        let mut f = Function::new(locals);
        for stmt in self.code.clone() {
            self.compile_stmt(&mut f, stmt);
        }
        f.instruction(&Instruction::End);
        codes.function(&f);
        for i in self.kys_funcs.clone() {
            if let Stmt::Fn {
                name: _,
                params,
                body,
                return_type: _,
            } = i {
                let mut locals = vec![];
                match *body.clone() {
                    Stmt::Block {
                        vars,
                        ..
                    } => {
                        for var in vars {
                            match var {
                                TokenType::Int => locals.push((1, ValType::I64)),
                                TokenType::Float => locals.push((1, ValType::F64)),
                                TokenType::Bool => locals.push((1, ValType::I32)),
                                TokenType::String => locals.push((1, ValType::I32)),
                                _ => self.error("unreachable?"),
                            }
                        }
                    }
                    _ => self.error("unreachable?"),
                }
                let mut f = Function::new(locals);
                for param in params {
                    self.vars.insert(match param.1.literal.clone().unwrap() {
                        Value::String(s) => s,
                        _ => panic!("unreachable?"),
                    }, (self.vars_count, param.0));
                    self.vars_count += 1;
                }
                self.compile_stmt(&mut f, *body.clone());
                f.instruction(&Instruction::End);
                codes.function(&f);
            }
        }
        self.module.section(&codes);

        let mut data = DataSection::new();
        for (&offset, length) in &self.offsets {
            data.active(0, &ConstExpr::i32_const(offset), self.strings[offset as usize..(offset + length) as usize].into_iter().copied());
        }
        self.module.section(&data);

        let wasm_bytes = self.module.clone().finish();
        let mut validator = Parser::new(0);
        if !self.path.ends_with(".wasm") {
            self.path = "output.wasm".to_string();
        }
        fs::write(&self.path, &wasm_bytes).expect("Failed to write Wasm to file");
        if is_wat {
            fs::write(&self.path.replace(".wasm", ".wat"), wasmprinter::print_file(&self.path).unwrap()).expect("Failed to write Wat to file");
        }
    }

    fn compile_stmt(&mut self, function: &mut Function, stmt: Stmt) {
        match stmt {
            Stmt::Print(expr) => {
                let t = self.compile_str(function, expr); // allow ints + strings, precomputed in rust, add to string hasmap!!
                self.print_wasm(function, t);
            }
            Stmt::Block{
                stmts,
                vars: _,
            } => {
                for stmt in stmts {
                    self.compile_stmt(function, stmt);
                }
            }
            Stmt::Expression(expr) => {
                self.compile_expr(function, expr);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let t = self.compile_expr(function, condition);
                //check that the condition is a boolean
                if let Value::Bool(_) = t {
                } else {
                    self.error("condition must evaluate to a boolean");
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                self.compile_stmt(function, *then_branch);
                if let Some(else_branch) = else_branch {
                    function.instruction(&Instruction::Else);
                    self.compile_stmt(function, *else_branch);
                }
                function.instruction(&Instruction::End);
            }
            Stmt::Var {
                value,
                name,
                t,
            } => {
                if let Some(value) = value {
                    let val1 = self.compile_expr(function, value);
                    let s = val1.as_str();
                    let len = s.len() as i32;
                    let index = self.make_string(s);
                    self.string_vars.insert(name.literal.clone().unwrap().as_str().to_string(), index);
                    match val1.clone() {
                        Value::Int(_) => {if t != TokenType::Int {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
                        Value::Float(_) => {if t != TokenType::Float {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
                        Value::Bool(_) => {if t != TokenType::Bool {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
                        Value::String(_) => {if t != TokenType::String {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
                        Value::Index(i) => {
                            if t != TokenType::String {
                                self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());
                            } else {
                                function.instruction(&Instruction::I32Const(i));
                                self.string_vars.insert(name.literal.clone().unwrap().as_str().to_string(), i);
                            }
                        },
                    }
                } else {
                    match t {
                        TokenType::Int => {function.instruction(&Instruction::I64Const(0));},
                        TokenType::Float => {function.instruction(&Instruction::F64Const(0.0));},
                        TokenType::Bool => {function.instruction(&Instruction::I32Const(0));},
                        TokenType::String => {function.instruction(&Instruction::I32Const(0));},
                        _ => self.error("unreachable?"),
                    }
                }
                if self.vars.contains_key(&name.literal.clone().unwrap().as_str()) {
                    self.error(format!("variable \"{}\" already declared", name.literal.clone().unwrap().as_str()).as_str());
                }
                self.vars.insert(name.literal.clone().unwrap().as_str().to_string(), (self.vars_count, t));
                self.vars_count += 1;
                function.instruction(&Instruction::LocalSet(self.vars.get(&name.literal.unwrap().as_str()).unwrap().0));
            }
            Stmt::While {
                condition,
                block,
            } => {
                let t = self.compile_expr(function, condition.clone());
                //check that the condition is a boolean
                if t != Value::Bool(true) {
                    self.error("condition must evaluate to a boolean");
                }
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                self.compile_stmt(function, *block);
                self.compile_expr(function, condition);
                function.instruction(&Instruction::BrIf(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            Stmt::Return{
                returnee,
                return_type,
            } => {
                match self.compile_expr(function, returnee) {
                    Value::String(_) => {
                        if return_type != TokenType::String {
                            self.error("type mismatch, cannot return string");
                        }
                    }
                    Value::Float(_) => {
                        if return_type != TokenType::Float {
                            self.error("type mismatch, cannot return float");
                        }
                    }
                    Value::Int(_) => {
                        if return_type != TokenType::Int {
                            self.error("type mismatch, cannot return int");
                        }
                    }
                    Value::Bool(_) => {
                        if return_type != TokenType::Bool {
                            self.error("type mismatch, cannot return bool");
                        }
                    }
                    Value::Index(_) => {
                        if return_type != TokenType::String {
                            self.error("type mismatch, cannot return string");
                        }
                    }
                }
                function.instruction(&Instruction::Return);
            }
            Stmt::Break(n) => {
                function.instruction(&Instruction::Br(n as u32 + 1));
            }
            _ => self.error("unreachable?"),
        }
    }

    fn compile_expr(&mut self, function: &mut Function, expr: Expr) -> Value {
        match expr {
            Expr::Grouping(expr) => self.compile_expr(function, *expr),
            Expr::Literal(val) => {
                return match val {
                    Value::Int(n) => {
                        function.instruction(&Instruction::I64Const(n));
                        Value::Int(n)
                    },
                    Value::Float(n) => {
                        function.instruction(&Instruction::F64Const(n));
                        Value::Float(n)
                    },
                    Value::Bool(b) => {
                        function.instruction(&Instruction::I32Const(if b { 1 } else { 0 }));
                        Value::Bool(b)
                    },
                    Value::String(s) => {
                        Value::Index(self.make_string(s))
                    },
                    _ => {self.error("unreachable?"); Value::Int(0)}
                }
            },
            Expr::Assign {
                name,
                value,
            } => {
                let val = self.compile_expr(function, *value);
                match self.vars.get(&name.literal.clone().unwrap().as_str()).unwrap().1 {
                    TokenType::Int => {
                        if let Value::Int(_) = val {
                        } else {
                            self.error(format!("Cannot assign non-integer value to variable \"{}\" of type Int", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    TokenType::Float => {
                        if let Value::Float(_) = val {
                        } else {
                            self.error(format!("Cannot assign non-float value to variable \"{}\" of type Float", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    TokenType::Bool => {
                        if let Value::Bool(_) = val {
                        } else {
                            self.error(format!("Cannot assign non-boolean value to variable \"{}\" of type Bool", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    TokenType::String => {
                        if let Value::String(_) = val {
                        } else {
                            self.error(format!("Cannot assign non-string value to variable \"{}\" of type String", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    _ => self.error("unreachable?"),
                }
                function.instruction(&Instruction::LocalSet(self.vars.get(&name.literal.clone().unwrap().as_str()).unwrap().0));
                let s = val.as_str();
                let len = s.len() as i32;
                let index = self.make_string(s);
                self.string_vars.insert(name.literal.clone().unwrap().as_str().to_string(), index);
                Value::Int(0)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let t1 = self.compile_expr(function, *left);
                let t2 = self.compile_expr(function, *right);
                self.bin(function, &t1, &t2, operator.tt)
            }
            Expr::Variable(t) => {
                function.instruction(&Instruction::LocalGet(self.vars.get(&t.literal.clone().unwrap_or_else(|| {
                    self.error(&format!("cannot get variable {}, perhaps it contains a function call?", t.literal.clone().unwrap().as_str()));
                    panic!()
                }).as_str()).unwrap_or_else(|| {
                    self.error(&format!("cannot get variable {}, perhaps it contains a function call?", t.literal.clone().unwrap().as_str()));
                    panic!()
                }).0));
                match self.vars.get(&t.literal.clone().unwrap_or_else(|| {
                    self.error(&format!("cannot get variable {}, perhaps it contains a function call?", t.literal.clone().unwrap().as_str()));
                    panic!()
                }).as_str()).unwrap_or_else(|| {
                    self.error(&format!("cannot get variable {}, perhaps it contains a function call?", t.literal.clone().unwrap().as_str()));
                    panic!()
                }).1 {
                    TokenType::Int => Value::Int(0),
                    TokenType::Float => Value::Float(0.0),
                    TokenType::Bool => Value::Bool(true),
                    TokenType::String => Value::String("".to_owned()),
                    _ => {self.error("unreachable?"); Value::Int(0)}
                }
            }
            Expr::Unary {
                operator,
                expression
            } => {
                let t1 = self.compile_expr(function, *expression);
                self.unary(function, &t1, operator.tt);
                t1
            }
            Expr::Call {
                callee,
                arguments
            } => {
                for arg in arguments {
                    self.compile_expr(function, arg);
                }
                match *callee {
                    Expr::Variable(t) => {
                        function.instruction(&Instruction::Call(self.funcs.get(&t.literal.clone().unwrap().as_str()).unwrap().0));
                        match self.funcs.get(&t.literal.clone().unwrap().as_str()).unwrap().1 {
                            TokenType::Int => Value::Int(0),
                            TokenType::Float => Value::Float(0.0),
                            TokenType::Bool => Value::Bool(true),
                            TokenType::String => Value::String("".to_owned()),
                            _ => {self.error("unreachable?"); Value::Int(0)}
                        }
                    }
                    _ => {self.error("unreachable?"); Value::Int(0)},
                }
            }
        }
    }

    fn compile_str(&mut self, function: &mut Function, expr: Expr) -> i32 {
        match expr {
            Expr::Grouping(expr) => {self.compile_str(function, *expr)},
            Expr::Binary {
                left,
                right,
                ..
            } => {
                let t1 = self.compile_str(function, *left);
                let t2 = self.compile_str(function, *right);
                self.add_strings(function, t1, t2)
            }
            Expr::Literal(val) => {
                self.make_string(val.as_str())
            }
            Expr::Variable(t) => {
                return self.string_vars.get(&t.literal.clone().unwrap().as_str()).unwrap_or_else(|| {
                    self.error(&format!("cannot stringify variable {}", t.literal.clone().unwrap().as_str()));
                    panic!()
                }).clone();
            }
            Expr::Call {
                callee,
                arguments,
            } => {
                for arg in arguments {
                    self.compile_expr(function, arg);
                }
                match *callee {
                    Expr::Variable(t) => {
                        // let func_num = self.funcs.get(&t.literal.clone().unwrap().as_str()).unwrap().0;
                        // function.instruction(&Instruction::Call(self.funcs.get(&t.literal.clone().unwrap().as_str()).unwrap().0));
                        // self.precompute_string(self.kys_funcs[func_num as usize - 2].clone())
                        self.error("we are sorry, keyscript does not support printing function calls yet. (blame wasm's shitty strings). please print the return value inside the function before returning it."); 0
                    }
                    _ => {self.error("unreachable?"); 0},
                }
            }
            _ => {self.error(format!("idk what else: {:?}", expr).as_str()); 0}
        }
    }

    // fn precompute_string(&mut self, function: Stmt) -> i32 {
    //     //todo: HELP :sob:
    //     //use self.
    //     0
    // }

    fn bin(&mut self, function: &mut Function, t1: &Value, t2: &Value, operator: TokenType) -> Value {
        match (t1, t2) {
            (Value::Int(_), Value::Int(_)) => {
                match operator {
                    TokenType::Plus => {function.instruction(&Instruction::I64Add); Value::Int(0)},
                    TokenType::Minus => {function.instruction(&Instruction::I64Sub); Value::Int(0)},
                    TokenType::Star => {function.instruction(&Instruction::I64Mul); Value::Int(0)},
                    TokenType::Slash => {function.instruction(&Instruction::I64DivU); Value::Int(0)},
                    TokenType::EqualEqual => {function.instruction(&Instruction::I64Eq); Value::Bool(true)},
                    TokenType::BangEqual => {function.instruction(&Instruction::I64Ne); Value::Bool(true)},
                    TokenType::Less => {function.instruction(&Instruction::I64LtU); Value::Bool(true)},
                    TokenType::LessEqual => {function.instruction(&Instruction::I64LeU); Value::Bool(true)},
                    TokenType::Greater => {function.instruction(&Instruction::I64GtU); Value::Bool(true)},
                    TokenType::GreaterEqual => {function.instruction(&Instruction::I64GeU); Value::Bool(true)},
                    TokenType::Modulo => {function.instruction(&Instruction::I64RemU); Value::Int(0)},
                    _ => {self.error("unreachable?"); Value::Bool(true)},
                }
            }
            (Value::Float(_), Value::Float(_)) => {
                match operator {
                    TokenType::Plus => {function.instruction(&Instruction::F64Add); Value::Float(0.0)},
                    TokenType::Minus => {function.instruction(&Instruction::F64Sub); Value::Float(0.0)},
                    TokenType::Star => {function.instruction(&Instruction::F64Mul); Value::Float(0.0)},
                    TokenType::Slash => {function.instruction(&Instruction::F64Div); Value::Float(0.0)},
                    TokenType::EqualEqual => {function.instruction(&Instruction::F64Eq); Value::Bool(true)},
                    TokenType::BangEqual => {function.instruction(&Instruction::F64Ne); Value::Bool(true)},
                    TokenType::Less => {function.instruction(&Instruction::F64Lt); Value::Bool(true)},
                    TokenType::LessEqual => {function.instruction(&Instruction::F64Le); Value::Bool(true)},
                    TokenType::Greater => {function.instruction(&Instruction::F64Gt); Value::Bool(true)},
                    TokenType::GreaterEqual => {function.instruction(&Instruction::F64Ge); Value::Bool(true)},
                    _ => {self.error("undefined operation"); Value::Bool(true)},
                }
            }
            (Value::Index(s), Value::Index(s1)) => {
                return Value::Index(self.add_strings(function, *s, *s1));
            }
            (Value::Int(_), _) => {
                {self.error("Cannot execute this operation on different types, use 2 ints"); Value::Bool(true)}
            }
            (Value::Float(_), _) => {
                {self.error("Cannot execute this operation on different types, use 2 floats"); Value::Bool(true)}
            }
            (Value::Bool(_), Value::Bool(_)) => {
                match operator {
                    TokenType::EqualEqual => {function.instruction(&Instruction::I32Eq); Value::Bool(true)},
                    TokenType::BangEqual => {function.instruction(&Instruction::I32Ne); Value::Bool(true)},
                    TokenType::And => {function.instruction(&Instruction::I32And); Value::Bool(true)},
                    TokenType::Or => {function.instruction(&Instruction::I32Or); Value::Bool(true)},
                    _ => {self.error("cannot use operation on 2 booleans"); Value::Bool(true)}
                }
            }
            (Value::Bool(_), _) => {
                {self.error("Cannot execute this operation on different types, use 2 booleans"); Value::Bool(true)}
            }
            _ => {self.error("undefined operation"); Value::Bool(true)}
        }
    }

    fn unary(&self, function: &mut Function, t1: &Value, operator: TokenType) {
        match t1 {
            Value::Int(_) => {
                match operator {
                    TokenType::Minus => {
                        function.instruction(&Instruction::I64Const(-1));
                        function.instruction(&Instruction::I64Mul);
                    },
                    _ => self.error("undefined unary operation for type int"),
                };
            }
            Value::Float(_) => {
                match operator {
                    TokenType::Minus => {function.instruction(&Instruction::F64Neg);},
                    _ => self.error("undefined unary operation for type float"),
                };
            }
            Value::Bool(_) => {
                match operator {
                    TokenType::Bang => {function.instruction(&Instruction::I64Eqz);},
                    _ => self.error("undefined unary operation for type boolean"),
                };
            }
            _ => self.error("Cannot negate this type")
        }
    }

    fn add_strings(&mut self, function: &mut Function, t1: i32, t2: i32) -> i32 {
        //takes 2 indexes to strings and return an index to the new string
        self.offsets.get(&t1).unwrap_or_else(|| {
            self.error("undefined string");
            panic!()
        });
        self.offsets.get(&t2).unwrap_or_else(|| {
            self.error("undefined string");
            panic!()
        });
        let mut s = String::new();
        for &t in &[t1, t2] {
            if let Some(offset) = self.offsets.get(&t) {
                for &c in &self.strings[t as usize..(t + offset) as usize] {
                    s.push(c as char);
                }
            }
        }
        self.make_string(s)
    }

    fn make_string(&mut self, s: String) -> i32 {
        let offset = self.strings.len();
        let b = s.as_bytes();
        self.offsets.insert(offset as i32, b.len() as i32);
        self.strings.extend_from_slice(b);
        offset as i32
    }

    fn print_wasm(&mut self, f: &mut Function, offset: i32) {
        let length = self.offsets.get(&offset).unwrap_or_else(|| {
            self.error("undefined string");
            panic!()
        });
        f.instruction(&Instruction::I32Const(offset));
        f.instruction(&Instruction::I32Const(*length));
        f.instruction(&Instruction::Call(0));
    }


    fn error(&self, msg: &str) {
        KeyScriptError::error(
            KeyScriptError::CompilerError,
            Some(msg),
            None,
            None); //todo: add line and filename
    }

}



