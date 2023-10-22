use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    Module, TypeSection, ValType,
};
use std::fs;
use crate::{ast::Expr, scanner::{Value, TokenType}};
use crate::ast::Stmt;
use crate::errors::KeyScriptError;

//REMINDER of wasm structure:
//first types
//then functions and indexes
//then exports
//then code
//booleans are stored in i32!!

pub fn wasm() {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    let params = vec![ValType::I32, ValType::I32];
    let results = vec![ValType::I32];
    types.function(params, results);
    module.section(&types);

    // Encode the function section.
    let mut functions = FunctionSection::new();
    let type_index = 0;
    functions.function(type_index);
    module.section(&functions);

    // Encode the export section with the function named "add".
    let mut exports = ExportSection::new();
    exports.export("add", ExportKind::Func, 0);
    module.section(&exports);

    // Encode the code section.
    let mut codes = CodeSection::new();
    let locals = vec![];
    let mut f = Function::new(locals);
    f.instruction(&Instruction::LocalGet(0));
    f.instruction(&Instruction::LocalGet(1));
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::End);
    codes.function(&f);
    module.section(&codes);


    //output:
    let wasm_bytes = module.finish();
    fs::write("output.wasm", &wasm_bytes).expect("Failed to write Wasm to file");
    fs::write("output.wat", wasmprinter::print_file("./output.wasm").unwrap()).expect("Failed to write Wat to file");
}

pub struct Compiler {
    pub module: Module,
    pub vars: HashMap<String, u32>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            module: Module::new(),
            vars: HashMap::new(),
        }
    }

    pub fn compile(&mut self, expr: Expr) {
        let mut module = Module::new();

        let mut types = TypeSection::new();
        let params = vec![];
        let results = vec![ValType::F64];
        types.function(params, results);
        module.section(&types);

        let mut functions = FunctionSection::new();
        let type_index = 0;
        functions.function(type_index);
        module.section(&functions);

        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let locals = vec![];
        let mut f = Function::new(locals);
        self.compile_expr(&mut f, expr);
        f.instruction(&Instruction::End);
        codes.function(&f);
        module.section(&codes);

        let wasm_bytes = module.finish();
        fs::write("output.wasm", &wasm_bytes).expect("Failed to write Wasm to file");
        fs::write("output.wat", wasmprinter::print_file("./output.wasm").unwrap()).expect("Failed to write Wat to file");
    }

    fn compile_expr(&mut self, function: &mut Function,expr: Expr) -> Value {
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
            // Expr::Assign {
            //     name,
            //     value,
            // } => {
            //     self.compile_expr(function, *value);
            //     function.instruction(&Instruction::LocalSet(match name.literal.unwrap() {
            //         Value::String(s) => s,
            //         _ => panic!("kys"),
            //     }));
            //     Value::Int(0)
            // } todo: cant be used until we know in which variable name is each number (will be set in var declaration)
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let t1 = self.compile_expr(function, *left);
                let t2 = self.compile_expr(function, *right);
                self.bin(function, &t1, &t2, operator.tt);
                t1
            }
            // Expr::Variable(t) => {
            //     function.instruction(&Instruction::LocalGet(match t.literal.unwrap() {
            //         Value::String(s) => s,
            //         _ => panic!("kys"),
            //     }));
            //     Value::Int(0)
            // } todo: cant be used until we know in which variable name is each number (will be set in var declaration)
            Expr::Unary {
                operator,
                expression
            } => {
                let t1 = self.compile_expr(function, *expression);
                self.unary(function, &t1, operator.tt);
                t1
            }
            _ => Value::Int(0)
        }
    }

    fn bin(&self, function: &mut Function, t1: &Value, t2: &Value, operator: TokenType) {
        match (t1, t2) {
            (Value::Int(n1), Value::Int(n2)) => {
                match operator {
                    TokenType::Plus => {function.instruction(&Instruction::I64Add);},
                    TokenType::Minus => {function.instruction(&Instruction::I64Sub);},
                    TokenType::Star => {function.instruction(&Instruction::I64Mul);},
                    TokenType::Slash => {function.instruction(&Instruction::I64DivU);},
                    TokenType::EqualEqual => {function.instruction(&Instruction::I64Eq);},
                    TokenType::BangEqual => {function.instruction(&Instruction::I64Ne);},
                    TokenType::Less => {function.instruction(&Instruction::I64LtU);},
                    TokenType::LessEqual => {function.instruction(&Instruction::I64LeU);},
                    TokenType::Greater => {function.instruction(&Instruction::I64GtU);},
                    TokenType::GreaterEqual => {function.instruction(&Instruction::I64GeU);},
                    _ => self.error("unreachable?"),
                };
            }
            (Value::Float(n1), Value::Float(n2)) => {
                match operator {
                    TokenType::Plus => {function.instruction(&Instruction::F64Add);},
                    TokenType::Minus => {function.instruction(&Instruction::F64Sub);},
                    TokenType::Star => {function.instruction(&Instruction::F64Mul);},
                    TokenType::Slash => {function.instruction(&Instruction::F64Div);},
                    TokenType::EqualEqual => {function.instruction(&Instruction::F64Eq);},
                    TokenType::BangEqual => {function.instruction(&Instruction::F64Ne);},
                    TokenType::Less => {function.instruction(&Instruction::F64Lt);},
                    TokenType::LessEqual => {function.instruction(&Instruction::F64Le);},
                    TokenType::Greater => {function.instruction(&Instruction::F64Gt);},
                    TokenType::GreaterEqual => {function.instruction(&Instruction::F64Ge);},
                    _ => self.error("unreachable?"),
                };
            }
            (Value::Int(n1), _) => {
                self.error("Cannot execute this operation on different types, use 2 ints")
            }
            (Value::Float(n1), _) => {
                self.error("Cannot execute this operation on different types, use 2 floats")
            }
            (Value::Bool(n1), Value::Bool(n2)) => {
                match operator {
                    TokenType::EqualEqual => {function.instruction(&Instruction::I32Eq);},
                    TokenType::BangEqual => {function.instruction(&Instruction::I32Ne);},
                    TokenType::And => {function.instruction(&Instruction::I32And);},
                    TokenType::Or => {function.instruction(&Instruction::I32Or);},
                    _ => self.error("unreachable?"),
                };
            }
            _ => self.error("undefined operation")
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
                    _ => self.error("unreachable?"),
                };
            }
            Value::Float(n1) => {
                match operator {
                    TokenType::Minus => {function.instruction(&Instruction::F64Neg);},
                    _ => self.error("unreachable?"),
                };
            }
            Value::Bool(n1) => {
                match operator {
                    TokenType::Bang => {function.instruction(&Instruction::I64Eqz);},
                    _ => self.error("unreachable?"),
                };
            }
            _ => self.error("Cannot negate this type")
        }
    }


    fn error(&self, msg: &str) {
        KeyScriptError::error(
            KeyScriptError::RuntimeError,
            Some(msg),
            None,
            None); //todo: add line and filename
    }

}



