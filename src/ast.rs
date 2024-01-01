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
    FunCall(String, Box<Expr>),
}

#[derive(PartialEq, Debug)]
pub struct Function {
    pub name: String,
    pub arg_name: String,
    pub body_stmts: Vec<Expr>,
}
