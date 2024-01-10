use crate::ast;
use anyhow::{anyhow, Context, Result};
use inkwell::types::BasicType;
//use inkwell::values::AnyValue;
use inkwell::values::BasicValue;

pub struct CodeGen<'run, 'ictx: 'run> {
    ast: Vec<ast::Function>,
    context: &'ictx inkwell::context::Context,
    module: &'run inkwell::module::Module<'ictx>,
    builder: &'run inkwell::builder::Builder<'ictx>,
}

#[derive(Debug, Clone)]
enum LlvmValue<'ictx> {
    Int(inkwell::values::IntValue<'ictx>),
    // Values whose internal is unknown to Chiika. Handled as `i8*`
    Opaque(inkwell::values::PointerValue<'ictx>),
    Func(inkwell::values::FunctionValue<'ictx>, ast::FunTy),
    FuncPtr(inkwell::values::PointerValue<'ictx>, ast::FunTy),
}

impl<'ictx> LlvmValue<'ictx> {
    fn into_arg_value(self) -> inkwell::values::BasicValueEnum<'ictx> {
        match self {
            LlvmValue::Int(x) => x.into(),
            LlvmValue::Opaque(x) => x.into(),
            LlvmValue::Func(x, _) => x.as_global_value().as_basic_value_enum(),
            LlvmValue::FuncPtr(x, _) => x.into(),
        }
    }

    fn expect_int(self) -> Result<inkwell::values::IntValue<'ictx>> {
        match self {
            LlvmValue::Int(x) => Ok(x),
            _ => Err(anyhow!("expected int")),
        }
    }

    fn expect_func(self) -> Result<(inkwell::values::FunctionValue<'ictx>, ast::FunTy)> {
        match self {
            LlvmValue::Func(x, y) => Ok((x, y)),
            _ => Err(anyhow!("expected func")),
        }
    }
}

pub fn run(ast: Vec<ast::Declaration>) -> Result<()> {
    let (externs, funcs) = ast::Declaration::split(ast);

    let context = inkwell::context::Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();
    let code_gen = CodeGen::new(funcs, &context, &module, &builder);
    code_gen.gen_declares(&externs);
    code_gen.gen_program()?;
    code_gen
        .module
        .write_bitcode_to_path(std::path::Path::new("a.bc"));
    code_gen
        .module
        .print_to_file("a.ll")
        .map_err(|llvm_str| anyhow!("{}", llvm_str.to_string()))?;
    log("Generated a.bc and a.ll");
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

    fn llvm_int(&self, n: u64) -> LlvmValue<'ictx> {
        LlvmValue::Int(self.context.i64_type().const_int(n, false))
    }

    fn llvm_type(&self, ty: &ast::Ty) -> inkwell::types::BasicTypeEnum<'ictx> {
        match ty {
            ast::Ty::Raw(name) => match &name[..] {
                "any" => self.context.i8_type().ptr_type(Default::default()).into(),
                "int" => self.context.i64_type().into(),
                "$ENV" => self.context.i8_type().ptr_type(Default::default()).into(),
                "$FUTURE" => self.context.i8_type().ptr_type(Default::default()).into(),
                _ => panic!("unknown chiika-1 type `{:?}'", ty),
            },
            ast::Ty::Fun(ast::FunTy { param_tys, ret_ty }) => {
                let params = param_tys
                    .iter()
                    .map(|x| self.llvm_type(x).into())
                    .collect::<Vec<_>>();
                self.llvm_type(ret_ty)
                    .fn_type(&params, false)
                    .ptr_type(Default::default())
                    .into()
            }
        }
    }

    fn cast(
        &self,
        v: inkwell::values::BasicValueEnum<'ictx>,
        ty: &ast::Ty,
    ) -> Result<LlvmValue<'ictx>> {
        let cast = match ty {
            ast::Ty::Raw(name) => match &name[..] {
                "int" => LlvmValue::Int(v.try_into().map_err(|_| anyhow!("not int"))?),
                "$ENV" => {
                    LlvmValue::Opaque(v.try_into().map_err(|_| anyhow!("not {:?}: {:?}", ty, v))?)
                }
                _ => panic!("unknown chiika-1 type `{:?}'", ty),
            },
            ast::Ty::Fun(fun_ty) => LlvmValue::FuncPtr(
                v.try_into().map_err(|_| anyhow!("not func"))?,
                fun_ty.clone(),
            ),
        };
        Ok(cast)
    }

    #[allow(unreachable_code)]
    fn recast(&self, v: LlvmValue<'ictx>, ty: &ast::Ty) -> Result<LlvmValue<'ictx>> {
        let _cast = match (v, ty) {
            //(LlvmValue::Int(_), ast::Ty::Raw("Int")) => v.clone(),
            _ => return Err(anyhow!("todo")),
        };
        Ok(_cast)
    }

    fn gen_declares(&self, externs: &[ast::Extern]) {
        for ext in externs {
            let arg_types = ext
                .params
                .iter()
                .map(|param| self.llvm_type(&param.ty).into())
                .collect::<Vec<_>>();
            let func_type = self.llvm_type(&ext.ret_ty).fn_type(&arg_types, false);
            self.module.add_function(&ext.name, func_type, None);
        }
    }

    fn gen_program(&self) -> Result<()> {
        for func in &self.ast {
            self.create_func(func);
        }
        for func in &self.ast {
            log(format!("Compiling function {}", &func.name));
            self.gen_func(func)?;
        }
        Ok(())
    }

    fn create_func(&self, func: &ast::Function) {
        let func_type = self
            .context
            .i64_type()
            .fn_type(&[self.context.i64_type().into()], false);
        self.module.add_function(&func.name, func_type, None);
    }

    fn gen_func(&self, func: &ast::Function) -> Result<()> {
        let f = self.module.get_function(&func.name).unwrap();
        let block = self.context.append_basic_block(f, "start");
        self.builder.position_at_end(block);
        self.gen_stmts(func, &func.body_stmts)?;
        Ok(())
    }

    fn gen_stmts(&self, func: &ast::Function, stmts: &[ast::Expr]) -> Result<()> {
        for i in 0..stmts.len() {
            let v = self.gen_expr(func, &stmts[i])?;
            if i == stmts.len() - 1 {
                self.builder
                    .build_return(Some(&v.expect_int()?.as_basic_value_enum()));
            }
        }
        Ok(())
    }

    fn gen_expr(&self, func: &ast::Function, expr: &ast::Expr) -> Result<LlvmValue<'ictx>> {
        log(format!("- {:?}", expr));
        let v = match expr {
            ast::Expr::Number(n) => self.llvm_int(*n as u64),
            ast::Expr::VarRef(s) => {
                if let Some(idx) = func.params.iter().position(|param| param.name == *s) {
                    let param = &func.params[idx];
                    let f = self.module.get_function(&func.name).unwrap();
                    let v = f.get_nth_param(idx as u32).unwrap();
                    self.cast(v, &param.ty)?
                } else {
                    let f = self
                        .module
                        .get_function(s)
                        .context(format!("unknown variable or function '{}'", s))?;
                    LlvmValue::Func(f, func.fun_ty())
                }
            }
            ast::Expr::OpCall(op, lhs, rhs) => {
                let l = self.gen_expr(func, lhs)?.expect_int()?;
                let r = self.gen_expr(func, rhs)?.expect_int()?;
                LlvmValue::Int(match op {
                    ast::BinOp::Add => self.builder.build_int_add(l, r, "result"),
                    ast::BinOp::Sub => self.builder.build_int_sub(l, r, "result"),
                })
            }
            ast::Expr::FunCall(func_expr, arg_exprs) => {
                let (f, f_ty) = self.gen_expr(func, func_expr)?.expect_func()?;
                let args = arg_exprs
                    .iter()
                    .map(|expr| self.gen_expr(func, expr))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .map(|arg| arg.into_arg_value().into())
                    .collect::<Vec<_>>();
                let x = self
                    .builder
                    .build_direct_call(f, &args, "result")
                    .try_as_basic_value()
                    .unwrap_left();
                self.cast(x.as_basic_value_enum(), &f_ty.ret_ty)?
            }
            ast::Expr::Cast(expr, ty) => {
                let v = self.gen_expr(func, expr)?;
                self.recast(v, ty)?
            }
        };
        Ok(v)
    }
}

fn log(msg: impl Into<String>) {
    println!("{}", msg.into());
}
