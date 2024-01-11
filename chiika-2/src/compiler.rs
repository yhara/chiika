use crate::ast::{self, FunTy, Ty};
use std::collections::HashMap;

pub fn compile(ast: Vec<ast::Declaration>) -> Vec<ast::Declaration> {
    let sigs = gather_sigs(&ast);
    ast.into_iter()
        .flat_map(|decl| match decl {
            ast::Declaration::Extern(x) => vec![ast::Declaration::Extern(compile_extern(x))],
            ast::Declaration::Function(x) => compile_func(&sigs, x)
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

fn compile_extern(mut e: ast::Extern) -> ast::Extern {
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

fn compile_func(sigs: &HashMap<String, FunTy>, f: ast::Function) -> Vec<ast::Function> {
    vec![f]
}
