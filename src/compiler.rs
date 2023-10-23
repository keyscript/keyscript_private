use std::collections::HashMap;
use wasm_encoder::{BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module, TypeSection, ValType};
use std::fs;
use colored::Colorize;
use crate::{ast::Expr, scanner::{Value, TokenType}};
use crate::ast::Stmt;
use crate::errors::KeyScriptError;
use wasmparser::Parser;
use wasmparser::StorageType::Val;
use crate::scanner::Token;

//REMINDER of wasm structure:
//first types
//then functions and indexes
//then exports
//then code
//booleans are stored in i32!!

pub struct Compiler {
    module: Module,
    vars: HashMap<String, (u32, TokenType)>,
    vars_count: u32,
    funcs: HashMap<String, (u32, TokenType)>,
    funcs_count: u32,
    vars1: Vec<TokenType>,
    code: Vec<Stmt>,
}

impl Compiler {
    pub fn new(code: Vec<Stmt>, vars1: Vec<TokenType>) -> Compiler {
        Compiler {
            module: Module::new(),
            vars: HashMap::new(),
            vars_count: 0,
            funcs: HashMap::new(),
            funcs_count: 0,
            vars1,
            code
        }
    }

    pub fn compile(&mut self) {
        let mut kys_funcs: Vec<Stmt> = Vec::new();
        while let Some(stmt) = self.code.first() {
            match stmt {
                Stmt::Fn {
                    name,
                    return_type,
                    ..
                } => {
                    kys_funcs.push(self.code.remove(0));
                }
                _ => break,
            }
        }
        println!("{:?}", kys_funcs);
        println!("{:?}", self.code);
        let counter = 1;
        for i in kys_funcs.iter() {
            match i {
                Stmt::Fn {
                    name,
                    params,
                    body,
                    return_type,
                } => {
                    self.funcs.insert(name.clone(), (counter, return_type.clone()));
                }
                _ => self.error("unreachable?"),
            }
        }

        let mut types = TypeSection::new();
        let params = vec![];
        let results = vec![];
        types.function(params, results); //main function
        for i in kys_funcs.iter() {
            if let Stmt::Fn {
                name,
                params,
                body,
                return_type,
            } = i {
                let mut params1 = vec![];
                for param in params {
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
                    _ => panic!("unreachable?"),
                }
                types.function(params1, results1);
            }
        }
        self.module.section(&types);

        let mut functions = FunctionSection::new();
        let type_index = 0;
        functions.function(type_index);
        let mut counter = 1;
        for i in kys_funcs.iter() {
            functions.function(counter);
            counter += 1;
        }
        self.module.section(&functions);

        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        counter = 1;
        for i in kys_funcs.iter() {
            if let Stmt::Fn {
                name,
                params,
                body,
                return_type,
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
        for i in kys_funcs.iter() {
            if let Stmt::Fn {
                name,
                params,
                body,
                return_type,
            } = i {
                let mut locals = vec![];
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

        let wasm_bytes = self.module.clone().finish();
        let mut validator = Parser::new(0);
        if let Err(e) = validator.parse(&wasm_bytes, false) {
            println!("Validation error: {:?} please report to the devs!", e);
            std::process::exit(0);
        } else {
            println!("{}", "Validation successful!".green());
        }
        fs::write("output.wasm", &wasm_bytes).expect("Failed to write Wasm to file");
        fs::write("output.wat", wasmprinter::print_file("./output.wasm").unwrap()).expect("Failed to write Wat to file");
    }

    fn compile_stmt(&mut self, function: &mut Function, stmt: Stmt) {
        match stmt {
            Stmt::Print(expr) => {
                panic!("print isnt working yet") //todo learn how to print in wasm
            }
            Stmt::Block{
                stmts,
                vars,
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
                if t != Value::Bool(true) {
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
                    match self.compile_expr(function, value) {
                        Value::Int(n) => {if t != TokenType::Int {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
                        Value::Float(n) => {if t != TokenType::Float {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
                        Value::Bool(n) => {if t != TokenType::Bool {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
                        Value::String(n) => {if t != TokenType::String {self.error(format!("type mismatch, cannot assign to {:?} \"{}\"", t, name.literal.clone().unwrap().as_str()).as_str());}},
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
                if self.vars.contains_key(name.literal.clone().unwrap().as_str()) {
                    self.error(format!("variable \"{}\" already declared", name.literal.clone().unwrap().as_str()).as_str());
                }
                self.vars.insert(name.literal.clone().unwrap().as_str().to_string(), (self.vars_count, t));
                self.vars_count += 1;
                function.instruction(&Instruction::LocalSet(self.vars.get(name.literal.unwrap().as_str()).unwrap().0));
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
            Stmt::Return(expr) => { //todo, return types
                self.compile_expr(function, expr);
                function.instruction(&Instruction::Return);
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
                        Value::Int(0)
                    },
                    Value::Float(n) => {
                        function.instruction(&Instruction::F64Const(n));
                        Value::Float(0.0)
                    },
                    Value::Bool(b) => {
                        function.instruction(&Instruction::I32Const(if b { 1 } else { 0 }));
                        Value::Bool(true)
                    },
                    Value::String(s) => {
                        function.instruction(&Instruction::I32Const(s.len() as i32));
                        Value::String("".to_owned())
                    }, //todo figure out how strings work in wasm
                }
            },
            Expr::Assign {
                name,
                value,
            } => {
                let val = self.compile_expr(function, *value);
                match self.vars.get(name.literal.clone().unwrap().as_str()).unwrap().1 {
                    TokenType::Int => {
                        if val != Value::Int(0) {
                            self.error(format!("Cannot assign non-integer value to variable \"{}\" of type Int", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    TokenType::Float => {
                        if val != Value::Float(0.0) {
                            self.error(format!("Cannot assign non-float value to variable \"{}\" of type Float", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    TokenType::Bool => {
                        if val != Value::Bool(true) {
                            self.error(format!("Cannot assign non-boolean value to variable \"{}\" of type Bool", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    TokenType::String => {
                        if val != Value::String("".to_owned()) {
                            self.error(format!("Cannot assign non-string value to variable \"{}\" of type String", name.literal.clone().unwrap().as_str()).as_str());
                        }
                    },
                    _ => self.error("unreachable?"),
                }
                function.instruction(&Instruction::LocalSet(self.vars.get(name.literal.clone().unwrap().as_str()).unwrap().0));
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
                function.instruction(&Instruction::LocalGet(self.vars.get(t.literal.clone().unwrap().as_str()).unwrap().0));
                match self.vars.get(t.literal.clone().unwrap().as_str()).unwrap().1 {
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
                        function.instruction(&Instruction::Call(self.funcs.get(t.literal.clone().unwrap().as_str()).unwrap().0));
                        match self.funcs.get(t.literal.clone().unwrap().as_str()).unwrap().1 {
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

    fn bin(&self, function: &mut Function, t1: &Value, t2: &Value, operator: TokenType) -> Value {
        match (t1, t2) {
            (Value::Int(n1), Value::Int(n2)) => {
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
                    _ => {self.error("unreachable?"); Value::Bool(true)},
                }
            }
            (Value::Float(n1), Value::Float(n2)) => {
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
            (Value::Int(n1), _) => {
                {self.error("Cannot execute this operation on different types, use 2 ints"); Value::Bool(true)}
            }
            (Value::Float(n1), _) => {
                {self.error("Cannot execute this operation on different types, use 2 floats"); Value::Bool(true)}
            }
            (Value::Bool(n1), Value::Bool(n2)) => {
                match operator {
                    TokenType::EqualEqual => {function.instruction(&Instruction::I32Eq); Value::Bool(true)},
                    TokenType::BangEqual => {function.instruction(&Instruction::I32Ne); Value::Bool(true)},
                    TokenType::And => {function.instruction(&Instruction::I32And); Value::Bool(true)},
                    TokenType::Or => {function.instruction(&Instruction::I32Or); Value::Bool(true)},
                    _ => {self.error("cannot use operation on 2 booleans"); Value::Bool(true)}
                }
            }
            (Value::Bool(n1), _) => {
                {self.error("Cannot execute this operation on different types, use 2 booleans"); Value::Bool(true)}
            }
            _ => {self.error("undefined operation"); Value::Bool(true)}
        }
    }

    fn unary(&self, function: &mut Function, t1: &Value, operator: TokenType) {
        match t1 {
            Value::Int(n1) => {
                match operator {
                    TokenType::Minus => {
                        function.instruction(&Instruction::I64Const(-1));
                        function.instruction(&Instruction::I64Mul);
                    },
                    _ => self.error("undefined unary operation for type int"),
                };
            }
            Value::Float(n1) => {
                match operator {
                    TokenType::Minus => {function.instruction(&Instruction::F64Neg);},
                    _ => self.error("undefined unary operation for type float"),
                };
            }
            Value::Bool(n1) => {
                match operator {
                    TokenType::Bang => {function.instruction(&Instruction::I64Eqz);},
                    _ => self.error("undefined unary operation for type boolean"),
                };
            }
            _ => self.error("Cannot negate this type")
        }
    }


    fn error(&self, msg: &str) {
        KeyScriptError::error(
            KeyScriptError::CompilerError,
            Some(msg),
            None,
            None); //todo: add line and filename
    }

}



