use crate::scanner::{Token, Value};

#[derive(Clone, Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    }, //assignment
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    }, //binary operations (+, -, .., ==, !=, <=, ..)
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Grouping(Box<Expr>), // "(" expression ")"
    Literal(Value),
    Unary {
        operator: Token,
        expression: Box<Expr>,
    }, // ! or - (negate)
    Variable(Token),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Print(Value, usize),
    Block(Vec<Stmt>, (usize, usize)),
    Expression(Expr),
    If {
        condition: Expr,
        block: Box<Stmt>,
        else_block: Option<Box<Stmt>>,
        lines: (usize, Option<usize>),
    },
    Var {
        name: Token,
        value: Option<Expr>,
    },
    While {
        condition: Expr,
        block: Box<Stmt>,
        line: usize,
    },
    For {
        identifier: Token,
        iterable: Expr,
        block: Box<Stmt>,
        line: usize,
    },
    Fn {
        name: Token,
        params: Vec<Token>,
        body: Box<Stmt>,
    },
    Return(Option<Expr>, usize),
}
