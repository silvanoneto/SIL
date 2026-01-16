//! Code generation compartilhado entre JIT e AOT
//!
//! Traduz bytecode VSP para Cranelift IR

use crate::vsp::{VspResult, VspError};
use crate::vsp::bytecode::SilcFile;

#[cfg(feature = "jit")]
use cranelift::prelude::*;

/// Build função VSP (usado por JIT e AOT)
#[cfg(feature = "jit")]
pub fn build_vsp_function(
    ctx: &mut cranelift_codegen::Context,
    fn_builder_ctx: &mut FunctionBuilderContext,
    bytecode: &SilcFile,
) -> VspResult<()> {
    let mut builder = FunctionBuilder::new(&mut ctx.func, fn_builder_ctx);
    
    // Criar bloco de entrada
    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);
    
    // TODO: Implementar tradução de opcodes
    // Por enquanto, apenas retorna 0 (stub)
    compile_instructions_stub(&mut builder, bytecode)?;
    
    // Return 0 (success)
    let zero = builder.ins().iconst(types::I32, 0);
    builder.ins().return_(&[zero]);
    
    builder.finalize();
    
    Ok(())
}

/// Stub para compilação de instruções
/// TODO: Implementar tradução completa de opcodes VSP para IR
#[cfg(feature = "jit")]
fn compile_instructions_stub(builder: &mut FunctionBuilder, _bytecode: &SilcFile) -> VspResult<()> {
    // Placeholder: só retorna
    // Quando implementar, mapear cada Opcode para instruções Cranelift:
    //
    // match opcode {
    //     Opcode::Nop => { /* skip */ }
    //     Opcode::LoadImm { reg, val } => {
    //         let v = builder.ins().iconst(types::I64, val);
    //         // store in register offset
    //     }
    //     Opcode::Add { dst, src1, src2 } => {
    //         let v1 = builder.ins().load(...);
    //         let v2 = builder.ins().load(...);
    //         let result = builder.ins().iadd(v1, v2);
    //         builder.ins().store(..., result);
    //     }
    //     // ... outros opcodes
    // }
    
    let _ = builder; // Suprimir warning
    Ok(())
}

#[cfg(not(feature = "jit"))]
pub fn build_vsp_function(
    _ctx: &mut (),
    _fn_builder_ctx: &mut (),
    _bytecode: &SilcFile,
) -> VspResult<()> {
    Err(VspError::CompilationError(
        "Code generation requires 'jit' feature".to_string()
    ))
}
