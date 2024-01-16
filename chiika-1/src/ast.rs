#[derive(PartialEq, Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Number(i64),
    VarRef(String),
    OpCall(BinOp, Box<Expr>, Box<Expr>),
    FunCall(Box<Expr>, Vec<Expr>),
    Cast(Box<Expr>, Ty),
    Alloc(String),
    Assign(String, Box<Expr>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Declaration {
    Extern(Extern),
    Function(Function),
}

impl Declaration {
    pub fn split(mut decls: Vec<Declaration>) -> (Vec<Extern>, Vec<Function>) {
        let mut externs = vec![];
        let mut funcs = vec![];
        while let Some(decl) = decls.pop() {
            match decl {
                Declaration::Extern(x) => externs.push(x),
                Declaration::Function(x) => funcs.push(x),
            }
        }
        (externs, funcs)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Ty,
    pub body_stmts: Vec<Expr>,
}

impl Function {
    pub fn fun_ty(&self) -> FunTy {
        FunTy {
            param_tys: self.params.iter().map(|x| x.ty.clone()).collect::<Vec<_>>(),
            ret_ty: Box::new(self.ret_ty.clone()),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Extern {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Ty,
}

impl Extern {
    pub fn fun_ty(&self) -> FunTy {
        FunTy {
            param_tys: self.params.iter().map(|x| x.ty.clone()).collect::<Vec<_>>(),
            ret_ty: Box::new(self.ret_ty.clone()),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Param {
    pub ty: Ty,
    pub name: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Ty {
    Raw(String),
    Fun(FunTy),
}

impl Ty {
    pub fn fun(param_tys: Vec<Ty>, ret_ty: Ty) -> Ty {
        Ty::Fun(FunTy {
            param_tys,
            ret_ty: Box::new(ret_ty),
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunTy {
    pub param_tys: Vec<Ty>,
    pub ret_ty: Box<Ty>,
}
