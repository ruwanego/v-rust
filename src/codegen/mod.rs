use crate::parse::ast::{Expr, Program, Stmt};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        CodeGen {
            context,
            module,
            builder,
        }
    }

    pub fn generate(&self, program: &Program) {
        // Declare printf
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.i8_type().ptr_type(inkwell::AddressSpace::default());
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true); // true for variadic
        let printf_func = self.module.add_function("printf", printf_type, None);

        for func in &program.functions {
            let fn_type = i32_type.fn_type(&[], false);
            let function = self.module.add_function(&func.name, fn_type, None);
            let basic_block = self.context.append_basic_block(function, "entry");

            self.builder.position_at_end(basic_block);

            let mut env = HashMap::new();

            // Traverse the body statements
            for stmt in &func.body {
                self.generate_stmt(stmt, printf_func, &mut env);
            }

            // return 0;
            let zero = i32_type.const_int(0, false);
            self.builder.build_return(Some(&zero));
        }
    }

    fn generate_stmt(
        &self,
        stmt: &Stmt,
        printf_func: inkwell::values::FunctionValue<'ctx>,
        env: &mut HashMap<String, (inkwell::types::BasicTypeEnum<'ctx>, inkwell::values::PointerValue<'ctx>)>,
    ) {
        match stmt {
            Stmt::ExprStmt(expr) => {
                if let Expr::FunctionCall { name, args } = expr {
                    if name == "println" && args.len() == 1 {
                        let arg = &args[0];
                        if let Expr::StringLiteral(s) = arg {
                            let mut formatted = s.clone();
                            formatted.push('\n');
                            let global_str = self.builder.build_global_string_ptr(&formatted, "str").unwrap();
                            self.builder.build_call(printf_func, &[global_str.as_pointer_value().into()], "printf_call").unwrap();
                        } else if let Some(val) = self.generate_expr(arg, env) {
                            let fmt = if val.is_int_value() && val.into_int_value().get_type().get_bit_width() == 1 {
                                "%d\n" // bool
                            } else {
                                "%lld\n" // i64
                            };
                            let global_str = self.builder.build_global_string_ptr(fmt, "fmt").unwrap();
                            self.builder.build_call(printf_func, &[global_str.as_pointer_value().into(), val.into()], "printf_call").unwrap();
                        }
                    }
                } else {
                    self.generate_expr(expr, env);
                }
            }
            Stmt::VarDecl { name, is_mut: _, expr } => {
                if let Some(val) = self.generate_expr(expr, env) {
                    let alloca = self.builder.build_alloca(val.get_type(), name).unwrap();
                    self.builder.build_store(alloca, val).unwrap();
                    env.insert(name.clone(), (val.get_type(), alloca));
                }
            }
            Stmt::Assign { name, expr } => {
                if let Some((_, ptr)) = env.get(name) {
                    if let Some(val) = self.generate_expr(expr, env) {
                        self.builder.build_store(*ptr, val).unwrap();
                    }
                } else {
                    unreachable!("Sema should have caught undefined variable: {}", name);
                }
            }
        }
    }

    fn generate_expr(
        &self,
        expr: &Expr,
        env: &HashMap<String, (inkwell::types::BasicTypeEnum<'ctx>, inkwell::values::PointerValue<'ctx>)>,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        match expr {
            Expr::IntLiteral(i) => Some(self.context.i64_type().const_int(*i as u64, false).into()),
            Expr::BoolLiteral(b) => Some(self.context.bool_type().const_int(if *b { 1 } else { 0 }, false).into()),
            Expr::StringLiteral(_) => None, // standalone strings do nothing for now
            Expr::Identifier(name) => {
                if let Some((typ, ptr)) = env.get(name) {
                    let loaded = self.builder.build_load(*typ, *ptr, name).unwrap();
                    Some(loaded)
                } else {
                    unreachable!("Sema should have caught undefined variable: {}", name);
                }
            }
            Expr::Binary { op, left, right } => {
                use crate::parse::ast::BinaryOp;
                let l = self.generate_expr(left, env)?.into_int_value();
                let r = self.generate_expr(right, env)?.into_int_value();
                
                let res = match op {
                    BinaryOp::Add => self.builder.build_int_add(l, r, "addtmp").unwrap(),
                    BinaryOp::Sub => self.builder.build_int_sub(l, r, "subtmp").unwrap(),
                    BinaryOp::Mul => self.builder.build_int_mul(l, r, "multmp").unwrap(),
                    BinaryOp::Div => self.builder.build_int_signed_div(l, r, "divtmp").unwrap(),
                    BinaryOp::Mod => self.builder.build_int_signed_rem(l, r, "modtmp").unwrap(),
                    BinaryOp::Eq => self.builder.build_int_compare(inkwell::IntPredicate::EQ, l, r, "eqtmp").unwrap(),
                    BinaryOp::NotEq => self.builder.build_int_compare(inkwell::IntPredicate::NE, l, r, "neqtmp").unwrap(),
                    BinaryOp::Lt => self.builder.build_int_compare(inkwell::IntPredicate::SLT, l, r, "lttmp").unwrap(),
                    BinaryOp::LtEq => self.builder.build_int_compare(inkwell::IntPredicate::SLE, l, r, "ltetmp").unwrap(),
                    BinaryOp::Gt => self.builder.build_int_compare(inkwell::IntPredicate::SGT, l, r, "gttmp").unwrap(),
                    BinaryOp::GtEq => self.builder.build_int_compare(inkwell::IntPredicate::SGE, l, r, "gtetmp").unwrap(),
                    BinaryOp::And => self.builder.build_and(l, r, "andtmp").unwrap(),
                    BinaryOp::Or => self.builder.build_or(l, r, "ortmp").unwrap(),
                };
                Some(res.into())
            }
            Expr::Unary { op, expr } => {
                use crate::parse::ast::UnaryOp;
                let val = self.generate_expr(expr, env)?.into_int_value();
                let res = match op {
                    UnaryOp::Minus => self.builder.build_int_neg(val, "negtmp").unwrap(),
                    UnaryOp::Not => self.builder.build_not(val, "nottmp").unwrap(),
                };
                Some(res.into())
            }
            Expr::FunctionCall { .. } => None, // for now, func calls return nothing
        }
    }

    pub fn write_obj(&self, output_path: &Path) -> Result<(), String> {
        Target::initialize_x86(&InitializationConfig::default());
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).map_err(|e| e.to_string())?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        self.module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        self.module.set_triple(&target_triple);

        target_machine
            .write_to_file(&self.module, FileType::Object, output_path)
            .map_err(|e| e.to_string())
    }
}
