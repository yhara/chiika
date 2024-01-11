#[derive(PartialEq, Debug, Clone)]
pub enum Declaration {
    Extern(Extern),
    Function(Function),
}
#[derive(PartialEq, Debug, Clone)]
pub struct Extern {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Ty,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Ty,
    pub body_stmts: Vec<Expr>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Param {
    pub ty: Ty,
    pub name: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Ty {
    Raw(String),
    //Fun(FunTy),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Number(i64),
    VarRef(String),
    //OpCall(BinOp, Box<Expr>, Box<Expr>),
    FunCall(Box<Expr>, Vec<Expr>),
    //Cast(Box<Expr>, Ty),
}
