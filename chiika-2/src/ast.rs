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

pub fn to_source(ast: Vec<Declaration>) -> String {
    ast.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("")
}

impl std::fmt::Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Declaration::Extern(x) => write!(f, "{}", x),
            Declaration::Function(x) => write!(f, "{}", x),
        }
    }
}

impl std::fmt::Display for Extern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self.params.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
        write!(f, "extern {}({}) -> {};\n", 
               &self.name,
               params,
               &self.ret_ty)
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self.params.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
        write!(f, "func {}({}) -> {} {{\n", 
               &self.name,
               params,
               &self.ret_ty)?;
        for expr in &self.body_stmts {
            write!(f, "  {};\n", expr)?;
        }
        write!(f, "}}\n")
    }
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.ty, &self.name)
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Raw(s) => write!(f, "{}", s)
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{}", n),
            Expr::VarRef(s) => write!(f, "{}", s),
            Expr::FunCall(fexpr, arg_exprs) => {
                let args = arg_exprs.iter().map(|x| x.to_string())
                    .collect::<Vec<_>>().join(", ");
                write!(f, "{}({})", fexpr, args)
            }
        }
    }
}
