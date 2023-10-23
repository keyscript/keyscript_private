use crate::scanner::{Token, TokenType, Value};

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
    Print(Value),
    Block{stmts: Vec<Stmt>, vars: Vec<TokenType>},
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Var {
        name: Token,
        value: Option<Expr>,
        t: TokenType,
    },
    While {
        condition: Expr,
        block: Box<Stmt>,
    },
    Fn {
        name: String,
        params: Vec<(TokenType, Token)>,
        body: Box<Stmt>,
        return_type: TokenType,
    },
    Return(Expr),
}
