#[derive(PartialEq, Debug)]
pub enum BinOp {
    Add,
    Sub,
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    Number(i64),
    Ident(String),
    OpCall(BinOp, Box<Expr>, Box<Expr>),
    Lambda(String, Box<Expr>),
    FunCall(Box<Expr>, Box<Expr>),
}

#[derive(PartialEq, Debug)]
pub enum Stmt {
    Declaration(String, Expr),
}
