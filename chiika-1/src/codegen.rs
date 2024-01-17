use crate::ast;
use anyhow::{anyhow, Context, Result};
use inkwell::types::BasicType;
//use inkwell::values::AnyValue;
use inkwell::values::BasicValue;
use std::collections::HashMap;

pub struct CodeGen<'run, 'ictx: 'run> {
    ast: Vec<ast::Function>,
    signatures: HashMap<String, ast::FunTy>,
    context: &'ictx inkwell::context::Context,
    module: &'run inkwell::module::Module<'ictx>,
    builder: &'run inkwell::builder::Builder<'ictx>,
}

#[derive(Debug, Clone)]
enum LlvmValue<'ictx> {
    Int(inkwell::values::IntValue<'ictx>),
    Any(inkwell::values::IntValue<'ictx>),
    // Values whose internal is unknown to Chiika. Handled as `i8*`
    Opaque(inkwell::values::PointerValue<'ictx>),
    Func(inkwell::values::FunctionValue<'ictx>, ast::FunTy),
    FuncPtr(inkwell::values::PointerValue<'ictx>, ast::FunTy),
}

impl<'ictx> LlvmValue<'ictx> {
    fn into_arg_value(self) -> inkwell::values::BasicValueEnum<'ictx> {
        match self {
            LlvmValue::Int(x) => x.into(),
            LlvmValue::Any(x) => x.into(),
            LlvmValue::Opaque(x) => x.into(),
            LlvmValue::Func(x, _) => x.as_global_value().as_basic_value_enum(),
            LlvmValue::FuncPtr(x, _) => x.into(),
        }
    }

    fn into_integer(self, t: inkwell::types::IntType<'ictx>) -> inkwell::values::IntValue<'ictx> {
        let ptr = match self {
            LlvmValue::Int(x) | LlvmValue::Any(x) => return x,
            LlvmValue::Opaque(x) => x,
            LlvmValue::Func(x, _) => x.as_global_value().as_pointer_value(),
            LlvmValue::FuncPtr(x, _) => x,
        };
        ptr.const_to_int(t)
    }

    fn expect_int(self) -> Result<inkwell::values::IntValue<'ictx>> {
        match self {
            LlvmValue::Int(x) => Ok(x),
            _ => Err(anyhow!("expected int but got {:?}", self)),
        }
    }
}

pub fn run(ast: Vec<ast::Declaration>) -> Result<()> {
    let (externs, funcs) = ast::Declaration::split(ast);
    let sigs = gather_sigs(&externs, &funcs);

    let context = inkwell::context::Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();
    let code_gen = CodeGen::new(funcs, sigs, &context, &module, &builder);
    code_gen.gen_declares(&externs);
    code_gen.gen_program()?;
    log("Finished compilation.");
    code_gen
        .module
        .print_to_file("../a.ll")
        .map_err(|llvm_str| anyhow!("{}", llvm_str.to_string()))?;
    log("Wrote a.ll.");
    code_gen
        .module
        .write_bitcode_to_path(std::path::Path::new("../a.bc"));
    log("Wrote a.bc.");
    Ok(())
}

fn gather_sigs(externs: &[ast::Extern], funcs: &[ast::Function]) -> HashMap<String, ast::FunTy> {
    let tys = externs
        .iter()
        .map(|x| (x.name.clone(), x.fun_ty()))
        .chain(funcs.iter().map(|x| (x.name.clone(), x.fun_ty())));
    tys.collect()
}

impl<'run, 'ictx: 'run> CodeGen<'run, 'ictx> {
    fn new(
        ast: Vec<ast::Function>,
        signatures: HashMap<String, ast::FunTy>,
        context: &'ictx inkwell::context::Context,
        module: &'run inkwell::module::Module<'ictx>,
        builder: &'run inkwell::builder::Builder<'ictx>,
    ) -> CodeGen<'run, 'ictx> {
        CodeGen {
            ast,
            signatures,
            context,
            module,
            builder,
        }
    }

    fn llvm_int(&self, n: u64) -> LlvmValue<'ictx> {
        LlvmValue::Int(self.context.i64_type().const_int(n, false))
    }

    fn llvm_fn_type(&self, ty: &ast::FunTy) -> inkwell::types::FunctionType<'ictx> {
        let params = ty
            .param_tys
            .iter()
            .map(|x| self.llvm_type(x).into())
            .collect::<Vec<_>>();
        self.llvm_type(&ty.ret_ty).fn_type(&params, false)
    }

    fn llvm_type(&self, ty: &ast::Ty) -> inkwell::types::BasicTypeEnum<'ictx> {
        match ty {
            ast::Ty::Raw(name) => match &name[..] {
                "$any" => self.context.i64_type().into(),
                "int" => self.context.i64_type().into(),
                "$ENV" => self.context.i8_type().ptr_type(Default::default()).into(),
                "$FUTURE" => self.context.i8_type().ptr_type(Default::default()).into(),
                _ => panic!("unknown chiika-1 type `{:?}'", ty),
            },
            ast::Ty::Fun(x) => self.llvm_fn_type(x).ptr_type(Default::default()).into(),
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
                "$any" => LlvmValue::Any(v.try_into().map_err(|_| anyhow!("not int(any)"))?),
                "$ENV" | "$FUTURE" => {
                    LlvmValue::Opaque(v.try_into().map_err(|_| anyhow!("not {:?}: {:?}", ty, v))?)
                }
                _ => panic!("unknown chiika-1 type to cast: `{:?}', value: {:?}", ty, v),
            },
            ast::Ty::Fun(fun_ty) => LlvmValue::FuncPtr(
                v.try_into().map_err(|_| anyhow!("not func: {:?}", v))?,
                fun_ty.clone(),
            ),
        };
        Ok(cast)
    }

    /// Cast LlvmValue to `ty`
    fn recast(&self, v: LlvmValue<'ictx>, ty: &ast::Ty) -> Result<LlvmValue<'ictx>> {
        let vv = match (v, ty) {
            (LlvmValue::Any(n), ast::Ty::Fun(_)) => {
                let t = self.context.i8_type().ptr_type(Default::default());
                self.builder.build_int_to_ptr(n, t, "f").into()
            }
            (v, _) => {
                if *ty == ast::Ty::Raw("$any".to_string()) {
                    v.into_integer(self.context.i64_type()).into()
                } else {
                    v.into_arg_value()
                }
            }
        };
        self.cast(vv, ty)
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
        let func_type = self.llvm_fn_type(&func.fun_ty());
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
        let mut lvars = HashMap::new();
        for i in 0..stmts.len() {
            let v = self.gen_expr(func, &mut lvars, &stmts[i])?;
            if i == stmts.len() - 1 {
                self.builder.build_return(Some(&v.into_arg_value()));
            }
        }
        Ok(())
    }

    fn gen_expr(
        &self,
        func: &ast::Function,
        lvars: &mut HashMap<String, inkwell::values::PointerValue<'ictx>>,
        expr: &ast::Expr,
    ) -> Result<LlvmValue<'ictx>> {
        log(format!("- {:?}", expr));
        let v = match expr {
            ast::Expr::Number(n) => self.llvm_int(*n as u64),
            ast::Expr::VarRef(s) => {
                if let Some(idx) = func.params.iter().position(|param| param.name == *s) {
                    let param = &func.params[idx];
                    let f = self.module.get_function(&func.name).unwrap();
                    let v = f.get_nth_param(idx as u32).unwrap();
                    self.cast(v, &param.ty)?
                } else if let Some(ptr) = lvars.get(s) {
                    let n = self
                        .builder
                        .build_load(self.context.i64_type(), ptr.clone(), "n")
                        .into_int_value();
                    LlvmValue::Int(n)
                } else {
                    let f = self
                        .module
                        .get_function(s)
                        .context(format!("unknown variable or function '{}'", s))?;
                    let fun_ty = self
                        .signatures
                        .get(s)
                        .expect(&format!("function {} not found", s));
                    LlvmValue::Func(f, fun_ty.clone())
                }
            }
            ast::Expr::OpCall(op, lhs, rhs) => {
                let l = self.gen_expr(func, lvars, lhs)?.expect_int()?;
                let r = self.gen_expr(func, lvars, rhs)?.expect_int()?;
                LlvmValue::Int(match &op[..] {
                    "+" => self.builder.build_int_add(l, r, "result"),
                    "-" => self.builder.build_int_sub(l, r, "result"),
                    _ => {
                        let pred = match &op[..] {
                            "==" => inkwell::IntPredicate::EQ,
                            "!=" => inkwell::IntPredicate::NE,
                            "<" => inkwell::IntPredicate::SLT,
                            "<=" => inkwell::IntPredicate::SLE,
                            ">" => inkwell::IntPredicate::SGT,
                            ">=" => inkwell::IntPredicate::SGE,
                            _ => return Err(anyhow!("unknown binop `{}'", op)),
                        };
                        let i1 = self.builder.build_int_compare(pred, l, r, "result");
                        self.builder
                            .build_int_cast(i1, self.context.i64_type(), "cast")
                    }
                })
            }
            ast::Expr::FunCall(func_expr, arg_exprs) => {
                let args = arg_exprs
                    .iter()
                    .map(|expr| self.gen_expr(func, lvars, expr))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .map(|arg| arg.into_arg_value().into())
                    .collect::<Vec<_>>();
                let (x, fun_ty) = match self.gen_expr(func, lvars, func_expr)? {
                    LlvmValue::Func(f, fun_ty) => (
                        self.builder
                            .build_direct_call(f, &args, "result")
                            .try_as_basic_value()
                            .unwrap_left(),
                        fun_ty,
                    ),
                    LlvmValue::FuncPtr(fptr, fun_ty) => {
                        let ftype = self.llvm_fn_type(&fun_ty);
                        (
                            self.builder
                                .build_indirect_call(ftype, fptr, &args, "result")
                                .try_as_basic_value()
                                .unwrap_left(),
                            fun_ty,
                        )
                    }
                    _ => return Err(anyhow!("not a function: {:?}", expr)),
                };
                self.cast(x.as_basic_value_enum(), &fun_ty.ret_ty)?
            }
            ast::Expr::Cast(expr, ty) => {
                let v = self.gen_expr(func, lvars, expr)?;
                self.recast(v, ty)?
            }
            ast::Expr::Alloc(name) => {
                let ptr = self.builder.build_alloca(self.context.i64_type(), name);
                lvars.insert(name.clone(), ptr);
                self.llvm_int(0)
            }
            ast::Expr::Assign(name, rhs) => {
                let v = self.gen_expr(func, lvars, rhs)?;
                let ptr = lvars
                    .get(name)
                    .expect(&format!("unknown variable `{}'", name));
                self.builder.build_store(ptr.clone(), v.into_arg_value());
                self.llvm_int(0)
            }
        };
        Ok(v)
    }
}

fn log(msg: impl Into<String>) {
    println!("{}", msg.into());
}
