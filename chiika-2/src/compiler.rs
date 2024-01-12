use crate::ast::{self, FunTy, Ty};
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
struct Compiler {
    sigs: HashMap<String, ast::FunTy>,
    chapters: Vec<Chapter>,
}

#[derive(PartialEq, Debug)]
struct Chapter {
    stmts: Vec<ast::Expr>
}

pub fn compile(ast: Vec<ast::Declaration>) -> Vec<ast::Declaration> {
    let sigs = gather_sigs(&ast);
    let c = Compiler {
        sigs,
    };
    ast.into_iter()
        .flat_map(|decl| match decl {
            ast::Declaration::Extern(x) => vec![ast::Declaration::Extern(c.compile_extern(x))],
            ast::Declaration::Function(x) => c.compile_func(&sigs, x)
                .into_iter()
                .map(ast::Declaration::Function)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn gather_sigs(decls: &[ast::Declaration]) -> HashMap<String, ast::FunTy> {
    decls
        .iter()
        .map(|x| match x {
            ast::Declaration::Extern(x) => (x.name.clone(), x.fun_ty()),
            ast::Declaration::Function(x) => (x.name.clone(), x.fun_ty()),
        })
        .collect()
}

impl Compiler {
    fn compile_extern(&self, mut e: ast::Extern) -> ast::Extern {
        if e.is_async {
            e.is_async = false;
            e.params.insert(0, ast::Param::new(Ty::raw("$ENV"), "$env"));
            let fun_ty = FunTy {
                is_async: false, // chiika-1 does not have notion of asyncness
                param_tys: vec![Ty::raw("$ENV"), e.ret_ty],
                ret_ty: Box::new(Ty::raw("$FUTURE")),
            };
            e.params
                .insert(0, ast::Param::new(Ty::Fun(fun_ty), "$cont"));
            e.ret_ty = Ty::raw("$FUTURE");
        }
        e
    }

    fn compile_func(&mut self, sigs: &HashMap<String, FunTy>, f: ast::Function) -> Result<Vec<ast::Function>> {
        let chapters = vec![Chapter::new()];
        for expr in f.body_stmts {
            chapters.last().unwrap().stmts.push(self.compile_expr(expr)?);
        }
        vec![f]
    }

    fn compile_expr(&mut self, sigs: &HashMap<String, FunTy>, e: ast::Expr) -> Result<Vec<ast::Function>> {
        match e {
            ast::Expr::Number(_) => e,
            ast::Expr::VarRef(_) => e,
            ast::Expr::FunCall(fexpr, arg_exprs) => {
                if 
            }
        }

    }
}
