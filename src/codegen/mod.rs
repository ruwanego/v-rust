use crate::parser::ast::Program;
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
        for func in &program.functions {
            // For now, hardcode signature to `i32 main()`
            let i32_type = self.context.i32_type();
            let fn_type = i32_type.fn_type(&[], false);
            let function = self.module.add_function(&func.name, fn_type, None);
            let basic_block = self.context.append_basic_block(function, "entry");

            self.builder.position_at_end(basic_block);

            // return 0;
            let zero = i32_type.const_int(0, false);
            self.builder.build_return(Some(&zero));
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
