//! Cranelift implementation of the `codegen_traits` backend contract.
//!
//! Pure-Rust code generation: no system LLVM required. Objects are linked
//! with the platform linker (MSVC `link.exe` on Windows, `cc` on Unix).

#![forbid(unsafe_code)]

use codegen_traits::{BackendError, CodegenBackend};
use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature, Value};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use frontend::parse::ast::{BinaryOp, UnaryOp};
use frontend::sema::{CheckedExpr, CheckedExprKind, CheckedProgram, CheckedStmt};
use frontend::types::Type;
use std::collections::HashMap;
use std::path::Path;

/// Cranelift implementation of the abstract backend contract.
pub struct CraneliftBackend;

impl CodegenBackend for CraneliftBackend {
    fn compile(&self, program: &CheckedProgram, output: &Path) -> Result<(), BackendError> {
        let object_bytes = emit_object(program).map_err(BackendError::new)?;

        let obj_dir = tempfile::tempdir()
            .map_err(|e| BackendError::new(format!("Failed to create object temp dir: {e}")))?;
        let obj_path = obj_dir.path().join("output.o");
        std::fs::write(&obj_path, object_bytes)
            .map_err(|e| BackendError::new(format!("Failed to write object file: {e}")))?;

        link_object(&obj_path, output).map_err(BackendError::new)
    }
}

/// Lower a checked program to a native object file in memory.
pub fn emit_object(program: &CheckedProgram) -> Result<Vec<u8>, String> {
    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "true").map_err(|e| e.to_string())?;
    flag_builder.set("opt_level", "none").map_err(|e| e.to_string())?;
    let isa = cranelift_native::builder()
        .map_err(|e| format!("Host machine is not supported: {e}"))?
        .finish(settings::Flags::new(flag_builder))
        .map_err(|e| e.to_string())?;

    let builder = ObjectBuilder::new(isa, "v_module", cranelift_module::default_libcall_names())
        .map_err(|e| e.to_string())?;
    let mut module = ObjectModule::new(builder);
    let ptr_type = module.target_config().pointer_type();

    // printf is imported with a minimal signature; call sites use
    // call_indirect with a per-call signature to model C varargs.
    let mut printf_sig = module.make_signature();
    printf_sig.params.push(AbiParam::new(ptr_type));
    printf_sig.returns.push(AbiParam::new(types::I32));
    let printf_id = module
        .declare_function("printf", Linkage::Import, &printf_sig)
        .map_err(|e| e.to_string())?;

    let mut fn_builder_ctx = FunctionBuilderContext::new();

    for func in &program.functions {
        let mut ctx = module.make_context();
        ctx.func.signature.returns.push(AbiParam::new(types::I32));

        let func_id = module
            .declare_function(&func.name, Linkage::Export, &ctx.func.signature)
            .map_err(|e| e.to_string())?;

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fn_builder_ctx);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            builder.seal_block(entry);

            let mut gen = FunctionGen {
                module: &mut module,
                builder,
                printf_id,
                ptr_type,
                env: HashMap::new(),
            };

            for stmt in &func.body {
                gen.stmt(stmt)?;
            }

            let zero = gen.builder.ins().iconst(types::I32, 0);
            gen.builder.ins().return_(&[zero]);
            gen.builder.finalize();
        }

        module.define_function(func_id, &mut ctx).map_err(|e| e.to_string())?;
    }

    module.finish().emit().map_err(|e| e.to_string())
}

struct FunctionGen<'a> {
    module: &'a mut ObjectModule,
    builder: FunctionBuilder<'a>,
    printf_id: FuncId,
    ptr_type: cranelift_codegen::ir::Type,
    env: HashMap<String, Variable>,
}

impl FunctionGen<'_> {
    fn stmt(&mut self, stmt: &CheckedStmt) -> Result<(), String> {
        match stmt {
            CheckedStmt::ExprStmt { expr, span: _ } => {
                if let CheckedExprKind::FunctionCall { name, args } = &expr.kind {
                    if name == "println" && args.len() == 1 {
                        return self.println(&args[0]);
                    }
                }
                self.expr(expr)?;
                Ok(())
            }
            CheckedStmt::VarDecl { name, is_mut: _, typ: _, expr, span: _ } => {
                if let Some(val) = self.expr(expr)? {
                    let ty = self.builder.func.dfg.value_type(val);
                    let var = self.builder.declare_var(ty);
                    self.builder.def_var(var, val);
                    self.env.insert(name.clone(), var);
                }
                Ok(())
            }
            CheckedStmt::Assign { name, typ: _, expr, span: _ } => {
                let var = *self
                    .env
                    .get(name)
                    .unwrap_or_else(|| unreachable!("Sema should have caught: {name}"));
                if let Some(val) = self.expr(expr)? {
                    self.builder.def_var(var, val);
                }
                Ok(())
            }
        }
    }

    fn println(&mut self, arg: &CheckedExpr) -> Result<(), String> {
        if let CheckedExprKind::StringLiteral(s) = &arg.kind {
            let mut text = s.clone();
            text.push('\n');
            let ptr = self.string_ptr(&text)?;
            self.call_printf(&[ptr]);
            return Ok(());
        }

        let Some(val) = self.expr(arg)? else { return Ok(()) };
        if arg.typ == Type::Bool {
            let extended = self.builder.ins().uextend(types::I32, val);
            let fmt = self.string_ptr("%d\n")?;
            self.call_printf(&[fmt, extended]);
        } else {
            let fmt = self.string_ptr("%lld\n")?;
            self.call_printf(&[fmt, val]);
        }
        Ok(())
    }

    /// Call printf through `call_indirect` so each call site can carry its
    /// own argument list, matching C varargs.
    fn call_printf(&mut self, args: &[Value]) {
        let callee = self.module.declare_func_in_func(self.printf_id, self.builder.func);
        let addr = self.builder.ins().func_addr(self.ptr_type, callee);

        let mut sig = Signature::new(self.builder.func.signature.call_conv);
        for &arg in args {
            sig.params.push(AbiParam::new(self.builder.func.dfg.value_type(arg)));
        }
        sig.returns.push(AbiParam::new(types::I32));
        let sig_ref = self.builder.import_signature(sig);

        self.builder.ins().call_indirect(sig_ref, addr, args);
    }

    fn string_ptr(&mut self, text: &str) -> Result<Value, String> {
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);

        let data_id =
            self.module.declare_anonymous_data(false, false).map_err(|e| e.to_string())?;
        let mut desc = DataDescription::new();
        desc.define(bytes.into_boxed_slice());
        self.module.define_data(data_id, &desc).map_err(|e| e.to_string())?;

        let gv = self.module.declare_data_in_func(data_id, self.builder.func);
        Ok(self.builder.ins().global_value(self.ptr_type, gv))
    }

    fn expr(&mut self, expr: &CheckedExpr) -> Result<Option<Value>, String> {
        let val = match &expr.kind {
            CheckedExprKind::IntLiteral(i) => Some(self.builder.ins().iconst(types::I64, *i)),
            CheckedExprKind::BoolLiteral(b) => {
                Some(self.builder.ins().iconst(types::I8, i64::from(*b)))
            }
            CheckedExprKind::StringLiteral(_) => None,
            CheckedExprKind::Identifier(name) => {
                let var = *self
                    .env
                    .get(name)
                    .unwrap_or_else(|| unreachable!("Sema should have caught: {name}"));
                Some(self.builder.use_var(var))
            }
            CheckedExprKind::Binary { op, left, right } => {
                use cranelift_codegen::ir::condcodes::IntCC;
                let l = self.expr(left)?.ok_or("binary operand produced no value")?;
                let r = self.expr(right)?.ok_or("binary operand produced no value")?;
                let ins = self.builder.ins();
                let res = match *op {
                    BinaryOp::Add => ins.iadd(l, r),
                    BinaryOp::Sub => ins.isub(l, r),
                    BinaryOp::Mul => ins.imul(l, r),
                    BinaryOp::Div => ins.sdiv(l, r),
                    BinaryOp::Mod => ins.srem(l, r),
                    BinaryOp::Eq => ins.icmp(IntCC::Equal, l, r),
                    BinaryOp::NotEq => ins.icmp(IntCC::NotEqual, l, r),
                    BinaryOp::Lt => ins.icmp(IntCC::SignedLessThan, l, r),
                    BinaryOp::LtEq => ins.icmp(IntCC::SignedLessThanOrEqual, l, r),
                    BinaryOp::Gt => ins.icmp(IntCC::SignedGreaterThan, l, r),
                    BinaryOp::GtEq => ins.icmp(IntCC::SignedGreaterThanOrEqual, l, r),
                    BinaryOp::And => ins.band(l, r),
                    BinaryOp::Or => ins.bor(l, r),
                };
                Some(res)
            }
            CheckedExprKind::Unary { op, expr } => {
                let val = self.expr(expr)?.ok_or("unary operand produced no value")?;
                let res = match *op {
                    UnaryOp::Minus => self.builder.ins().ineg(val),
                    // Logical not on a bool held in I8: flip the low bit.
                    UnaryOp::Not => self.builder.ins().bxor_imm(val, 1),
                };
                Some(res)
            }
            CheckedExprKind::FunctionCall { .. } => None,
        };
        Ok(val)
    }
}

fn link_object(obj_path: &Path, output: &Path) -> Result<(), String> {
    let out = run_linker(obj_path, output)?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Linker failed:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

#[cfg(windows)]
fn run_linker(obj_path: &Path, output: &Path) -> Result<std::process::Output, String> {
    let target = format!("{}-pc-windows-msvc", std::env::consts::ARCH);
    let tool = cc::windows_registry::find_tool(&target, "link.exe")
        .ok_or("MSVC link.exe not found; install Visual Studio Build Tools")?;
    let mut cmd = tool.to_command();
    cmd.arg("/NOLOGO")
        .arg(obj_path)
        .arg(format!("/OUT:{}", output.display()))
        .arg("/SUBSYSTEM:CONSOLE")
        .args(["/DEFAULTLIB:libcmt", "/DEFAULTLIB:libucrt", "/DEFAULTLIB:libvcruntime"])
        .args(["/DEFAULTLIB:legacy_stdio_definitions", "/DEFAULTLIB:kernel32"]);
    cmd.output().map_err(|e| format!("Failed to execute link.exe: {e}"))
}

#[cfg(not(windows))]
fn run_linker(obj_path: &Path, output: &Path) -> Result<std::process::Output, String> {
    let mut missing = Vec::new();
    for linker in ["cc", "gcc", "clang"] {
        match std::process::Command::new(linker).arg(obj_path).arg("-o").arg(output).output() {
            Ok(out) => return Ok(out),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => missing.push(linker),
            Err(e) => return Err(format!("Failed to execute linker `{linker}`: {e}")),
        }
    }
    Err(format!("Failed to execute linker. Tried: {}", missing.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_object_for_empty_program() {
        let program = CheckedProgram { functions: vec![] };
        let bytes = emit_object(&program).expect("emit should succeed");
        assert!(!bytes.is_empty());
    }
}
