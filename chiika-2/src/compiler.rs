use crate::ast::{self, FunTy, Ty};
use crate::asyncness_check::gather_sigs;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(PartialEq, Debug)]
struct Compiler {
    sigs: HashMap<String, ast::FunTy>,
    chapters: VecDeque<Chapter>,
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
        sigs: gather_sigs(&ast)?,
        chapters: Default::default(),
    };
    let mut new_decls = vec![];
    for decl in ast {
        match decl {
            ast::Declaration::Extern(x) => {
                new_decls.push(ast::Declaration::Extern(c.compile_extern(x)))
            }
            ast::Declaration::Function(x) => {
                let mut split_funcs = c
                    .compile_func(x)?
                    .into_iter()
                    .map(ast::Declaration::Function)
                    .collect::<Vec<_>>();
                new_decls.append(&mut split_funcs);
            }
        }
    }
    Ok(new_decls)
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

    fn compile_func(&mut self, mut f: ast::Function) -> Result<Vec<ast::Function>> {
        self.chapters.clear();
        self.chapters.push_back(Chapter::new());
        for expr in f.body_stmts.drain(..).collect::<Vec<_>>() {
            let new_expr = self.compile_expr(&f, expr)?;
            self.chapters.back_mut().unwrap().stmts.push(new_expr);
        }

        if self.chapters.len() == 1 {
            // Has no async call; no modification needed
            Ok(vec![ast::Function {
                name: f.name,
                params: f.params,
                ret_ty: f.ret_ty,
                body_stmts: self.chapters.pop_front().unwrap().stmts,
            }])
        } else {
            let chaps = self.chapters.drain(..).collect();
            self.generate_split_funcs(f, chaps)
        }
    }

    fn generate_split_funcs(
        &mut self,
        orig_func: ast::Function,
        mut chapters: VecDeque<Chapter>,
    ) -> Result<Vec<ast::Function>> {
        let n_chapters = chapters.len();
        let mut i = 0;
        let mut last_chap_result_ty = None;
        let mut split_funcs = vec![];
        while let Some(chap) = chapters.pop_front() {
            let new_func = if i == 0 {
                ast::Function {
                    name: orig_func.name.clone(),
                    params: prepend_async_params(&orig_func.params, orig_func.ret_ty.clone()),
                    ret_ty: Ty::raw("$FUTURE"),
                    body_stmts: prepend_async_intro(&orig_func, chap.stmts),
                }
            } else {
                ast::Function {
                    name: chapter_func_name(&orig_func.name, i),
                    params: vec![
                        ast::Param::new(Ty::raw("$ENV"), "$env"),
                        ast::Param::new(last_chap_result_ty.unwrap(), "$async_result"),
                    ],
                    ret_ty: Ty::raw("$FUTURE"),
                    body_stmts: if i == n_chapters - 1 {
                        append_async_outro(&orig_func, chap.stmts, orig_func.ret_ty.clone())
                    } else {
                        chap.stmts
                    },
                }
            };
            i += 1;
            last_chap_result_ty = Some(chap.async_result_ty);
            split_funcs.push(new_func);
        }
        Ok(split_funcs)
    }

    fn compile_expr(&mut self, orig_func: &ast::Function, e: ast::Expr) -> Result<ast::Expr> {
        let new_e = match e {
            ast::Expr::Number(_) => e,
            ast::Expr::VarRef(ref name) => {
                if self.sigs.contains_key(name) {
                    e
                } else if self.chapters.len() == 1 {
                    // The variable is just there in the first chapter
                    e
                } else {
                    let idx = orig_func
                        .params
                        .iter()
                        .position(|x| x.name == *name)
                        .expect(&format!("unknown variable `{}'", name));
                    ast::Expr::FunCall(
                        Box::new(ast::Expr::var_ref("chiika_env_ref")),
                        vec![ast::Expr::var_ref("$env"), ast::Expr::Number(idx as i64)],
                    )
                }
            }
            ast::Expr::FunCall(fexpr, arg_exprs) => {
                let mut new_args = arg_exprs
                    .into_iter()
                    .map(|x| self.compile_expr(orig_func, x))
                    .collect::<Result<Vec<_>>>()?;
                let ast::Expr::VarRef(callee_name) = *fexpr else {
                    return Err(anyhow!("not a function: {:?}", fexpr));
                };
                let Some(fun_ty) = self.sigs.get(&callee_name) else {
                    return Err(anyhow!("unknown function: {:?}", callee_name));
                };
                if fun_ty.is_async {
                    new_args.insert(0, ast::Expr::var_ref("$env"));
                    new_args.insert(
                        1,
                        ast::Expr::var_ref(chapter_func_name(&orig_func.name, self.chapters.len())),
                    );
                    let cps_call =
                        ast::Expr::FunCall(Box::new(ast::Expr::VarRef(callee_name)), new_args);

                    // Change chapter here
                    let last_chapter = self.chapters.back_mut().unwrap();
                    last_chapter.stmts.push(cps_call);
                    last_chapter.async_result_ty = (*fun_ty.ret_ty).clone();
                    self.chapters.push_back(Chapter::new());

                    ast::Expr::VarRef("$async_result".to_string())
                } else {
                    ast::Expr::FunCall(Box::new(ast::Expr::VarRef(callee_name)), new_args)
                }
            }
            ast::Expr::Cast(_, _) => panic!("chiika-2 does not have cast operation"),
        };
        Ok(new_e)
    }
}

/// Prepend params for async
fn prepend_async_params(params: &[ast::Param], result_ty: Ty) -> Vec<ast::Param> {
    let mut new_params = params.to_vec();
    new_params.insert(0, ast::Param::new(Ty::raw("$ENV"), "$env"));

    let fun_ty = FunTy {
        is_async: false, // chiika-1 does not have notion of asyncness
        param_tys: vec![Ty::raw("$ENV"), result_ty],
        ret_ty: Box::new(Ty::raw("$FUTURE")),
    };
    new_params.insert(1, ast::Param::new(Ty::Fun(fun_ty), "$cont"));

    new_params
}

fn prepend_async_intro(orig_func: &ast::Function, mut stmts: Vec<ast::Expr>) -> Vec<ast::Expr> {
    let push_items = vec![ast::Expr::var_ref("$cont")].into_iter().chain(
        orig_func
            .params
            .iter()
            .map(|param| ast::Expr::var_ref(&param.name)),
    );

    let mut push_calls = push_items
        .map(|arg| {
            let cast = ast::Expr::Cast(Box::new(arg), ast::Ty::raw("$any"));
            ast::Expr::FunCall(
                Box::new(ast::Expr::var_ref("chiika_env_push")),
                vec![ast::Expr::var_ref("$env"), cast],
            )
        })
        .collect::<Vec<_>>();
    push_calls.append(&mut stmts);
    push_calls
}

fn append_async_outro(
    orig_func: &ast::Function,
    mut stmts: Vec<ast::Expr>,
    result_ty: Ty,
) -> Vec<ast::Expr> {
    let result_value = stmts.pop().unwrap();
    let n_pop = orig_func.params.len() + 1; // +1 for $cont
    let env_pop = ast::Expr::FunCall(
        Box::new(ast::Expr::var_ref("chiika_env_pop")),
        vec![ast::Expr::var_ref("$env"), ast::Expr::Number(n_pop as i64)],
    );
    let fun_ty = FunTy {
        is_async: false, // chiika-1 does not have notion of asyncness
        param_tys: vec![Ty::raw("$ENV"), result_ty],
        ret_ty: Box::new(Ty::raw("$FUTURE")),
    };
    let cast = ast::Expr::Cast(Box::new(env_pop), Ty::Fun(fun_ty));
    let call_cont = ast::Expr::FunCall(
        Box::new(cast),
        vec![ast::Expr::var_ref("$env"), result_value],
    );
    stmts.push(call_cont);
    stmts
}

/// Create name of generated function like `foo_1`
fn chapter_func_name(orig_name: &str, chapter_idx: usize) -> String {
    format!("{}_{}", orig_name, chapter_idx)
}
