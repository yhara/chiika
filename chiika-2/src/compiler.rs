use crate::ast::{self, FunTy, Ty};
use std::collections::HashMap;
use anyhow::{anyhow, Result};

#[derive(PartialEq, Debug)]
struct Compiler {
    sigs: HashMap<String, ast::FunTy>,
    chapters: Vec<Chapter>,
}

#[derive(PartialEq, Debug)]
struct Chapter {
    stmts: Vec<ast::Expr>,
    // The resulting type of the async function called with the last stmt
    async_result_ty: Ty,
}

impl Chapter {
    fn new() -> Chapter {
        Chapter {
            stmts: vec![],
            async_result_ty: Ty::raw("[BUG] async_result_ty not set"),
        }
    }
}

pub fn compile(ast: Vec<ast::Declaration>) -> Result<Vec<ast::Declaration>> {
    let mut c = Compiler {
        sigs: gather_sigs(&ast),
        chapters: Default::default(),
    };
    let mut new_decls = vec![];
    for decl in ast {
        match decl {
            ast::Declaration::Extern(x) => new_decls.push(ast::Declaration::Extern(c.compile_extern(x))),
            ast::Declaration::Function(x) => {
                let mut split_funcs = c.compile_func(x)?
                    .into_iter()
                    .map(ast::Declaration::Function)
                    .collect::<Vec<_>>();
                new_decls.append(&mut split_funcs);
            }
        }
    }
    Ok(new_decls)
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
            e.params = prepend_async_params(&e.params, e.ret_ty);
            e.ret_ty = Ty::raw("$FUTURE");
        }
        e
    }

    fn compile_func(&mut self, f: ast::Function) -> Result<Vec<ast::Function>> {
        self.chapters = vec![Chapter::new()];
        for expr in f.body_stmts {
            let new_expr = self.compile_expr(expr)?;
            self.chapters.last_mut().unwrap().stmts.push(new_expr);
        }

        if self.chapters.len() == 1 {
            // Has no async call; no modification needed
            Ok(vec![
                ast::Function {
                    name: f.name,
                    params: f.params,
                    ret_ty: f.ret_ty,
                    body_stmts: self.chapters.pop().unwrap().stmts,
                }
            ])
        } else {
            let orig_name = f.name;
            let mut split_funcs = vec![];
            for i in 0..self.chapters.len() {
                let new_func = if i == 0 {
                    ast::Function {
                        name: orig_name.clone(),
                        params: prepend_async_params(&f.params, f.ret_ty.clone()),
                        ret_ty: Ty::raw("$FUTURE"),
                        body_stmts: self.chapters[i].stmts,
                    }
                } else {
                    ast::Function {
                        name: chapter_func_name(&orig_name, i),
                        params: vec![
                            ast::Param::new(Ty::raw("$ENV"), "$env"),
                            ast::Param::new(self.chapters[i-1].async_result_ty, "$async_result"),
                        ],
                        ret_ty: Ty::raw("$FUTURE"),
                        body_stmts: self.chapters[i].stmts,
                    }
                };
                split_funcs.push(new_func);
            }
            Ok(split_funcs)
        }
    }

    fn compile_expr(&mut self, e: ast::Expr) -> Result<ast::Expr> {
        let new_e = match e {
            ast::Expr::Number(_) => e,
            ast::Expr::VarRef(_) => e,
            ast::Expr::FunCall(fexpr, arg_exprs) => {
                let ast::Expr::VarRef(fname) = *fexpr else {
                    return Err(anyhow!("not a function: {:?}", fexpr));
                };
                let Some(fun_ty) = self.sigs.get(&fname) else {
                    return Err(anyhow!("unknown function: {:?}", fname));
                };
                if fun_ty.is_async {
                    let cps_call = ast::Expr::FunCall(fexpr, vec![
                        //todo
                    ]);
                    let last_chapter = self.chapters.last().unwrap();
                    last_chapter.stmts.push(cps_call);
                    last_chapter.async_result_ty = (*fun_ty.ret_ty).clone();
                    self.chapters.push(Chapter::new());
                    ast::Expr::VarRef("$async_result".to_string())
                } else {
                    e
                }
            }
        };
        Ok(new_e)
    }
}

/// Prepend params for async
fn prepend_async_params(params: &[ast::Param], result_ty: Ty) -> Vec<ast::Param> {
    let new_params = params.to_vec();
    new_params.insert(0, ast::Param::new(Ty::raw("$ENV"), "$env"));

    let fun_ty = FunTy {
        is_async: false, // chiika-1 does not have notion of asyncness
        param_tys: vec![Ty::raw("$ENV"), result_ty],
        ret_ty: Box::new(Ty::raw("$FUTURE")),
    };
    new_params
        .insert(0, ast::Param::new(Ty::Fun(fun_ty), "$cont"));

    new_params
}

// Create name of generated function like `foo_1`
fn chapter_func_name(orig_name: &str, chapter_idx: usize) -> String {
    format!("{}_{}", orig_name, chapter_idx)
}
