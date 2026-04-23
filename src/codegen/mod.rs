use crate::parser::ast::{Expr, Program};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;
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

            // Traverse the body expressions
            for expr in &func.body {
                self.generate_expr(expr, printf_func);
            }

            // return 0;
            let zero = i32_type.const_int(0, false);
            self.builder.build_return(Some(&zero));
        }
    }

    fn generate_expr(&self, expr: &Expr, printf_func: inkwell::values::FunctionValue<'ctx>) {
        match expr {
            Expr::StringLiteral(_) => {
                // Not doing anything standalone for now
            }
            Expr::IntLiteral(_) => {
                // Not doing anything standalone for now
            }
            Expr::FunctionCall { name, args } => {
                if name == "println" && args.len() == 1 {
                    if let Expr::StringLiteral(s) = &args[0] {
                        let mut formatted = s.clone();
                        formatted.push('\n');
                        let global_str = self.builder.build_global_string_ptr(&formatted, "str").unwrap();
                        self.builder.build_call(printf_func, &[global_str.as_pointer_value().into()], "printf_call").unwrap();
                    }
                }
            }
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
