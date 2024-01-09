use crate::ast;
use anyhow::{anyhow, Context, Result};

pub struct CodeGen<'run, 'ictx: 'run> {
    ast: Vec<ast::Function>,
    context: &'ictx inkwell::context::Context,
    module: &'run inkwell::module::Module<'ictx>,
    builder: &'run inkwell::builder::Builder<'ictx>,
}

pub fn run(ast: Vec<ast::Function>) -> Result<()> {
    let context = inkwell::context::Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();
    let code_gen = CodeGen::new(ast, &context, &module, &builder);
    code_gen.gen_declares();
    code_gen.gen_program()?;
    code_gen
        .module
        .write_bitcode_to_path(std::path::Path::new("a.bc"));
    code_gen
        .module
        .print_to_file("a.ll")
        .map_err(|llvm_str| anyhow!("{}", llvm_str.to_string()))?;
    Ok(())
}

impl<'run, 'ictx: 'run> CodeGen<'run, 'ictx> {
    fn new(
        ast: Vec<ast::Function>,
        context: &'ictx inkwell::context::Context,
        module: &'run inkwell::module::Module<'ictx>,
        builder: &'run inkwell::builder::Builder<'ictx>,
    ) -> CodeGen<'run, 'ictx> {
        CodeGen {
            ast,
            context,
            module,
            builder,
        }
    }

    fn gen_declares(&self) {
        let func_type = self
            .context
            .void_type()
            .fn_type(&[self.context.i64_type().into()], false);
        self.module
            .add_function("chiika_start_tokio", func_type, None);
        self.module.add_function("print", func_type, None);
    }

    fn gen_program(&self) -> Result<()> {
        for func in &self.ast {
            self.gen_func(func)?;
        }
        Ok(())
    }

    fn gen_func(&self, func: &ast::Function) -> Result<()> {
        let func_type = self
            .context
            .i64_type()
            .fn_type(&[self.context.i64_type().into()], false);
        let f = self.module.add_function(&func.name, func_type, None);
        let block = self.context.append_basic_block(f, "start");
        //self.builder.build_unconditional_branch(block);
        self.builder.position_at_end(block);
        self.gen_stmts(func, &func.body_stmts)?;
        Ok(())
    }

    fn gen_stmts(&self, func: &ast::Function, stmts: &[ast::Expr]) -> Result<()> {
        for i in 0..stmts.len() {
            let v = self.gen_expr(func, &stmts[i])?;
            if i == stmts.len() - 1 {
                self.builder.build_return(Some(&v));
            }
        }
        Ok(())
    }

    fn gen_expr(
        &self,
        func: &ast::Function,
        expr: &ast::Expr,
    ) -> Result<inkwell::values::IntValue<'run>> {
        let v = match expr {
            ast::Expr::Number(n) => self.context.i64_type().const_int(*n as u64, false),
            ast::Expr::Ident(s) => {
                if *s != func.arg_name {
                    return Err(anyhow!("unknown variable '{}'", s));
                }
                let f = self.module.get_function(&func.name).unwrap();
                f.get_nth_param(0).unwrap().into_int_value()
            }
            ast::Expr::OpCall(op, lhs, rhs) => {
                let l = self.gen_expr(func, lhs)?;
                let r = self.gen_expr(func, rhs)?;
                match op {
                    ast::BinOp::Add => self.builder.build_int_add(l, r, "result"),
                    ast::BinOp::Sub => self.builder.build_int_sub(l, r, "result"),
                }
            }
            ast::Expr::FunCall(fname, arg_expr) => {
                let f = self
                    .module
                    .get_function(fname)
                    .context(format!("unknown function '{}'", fname))?;
                let args = vec![self.gen_expr(func, arg_expr)?.into()];
                self.builder.build_direct_call(f, &args, "result");
                self.context.i64_type().const_int(0, false)
            }
        };
        Ok(v)
    }
}
