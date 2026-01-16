# VSP JIT Quick Reference

## File Locations

```
src/vsp/jit/
├── mod.rs          - JitError definitions
├── compiler.rs     - VspJit::compile()
└── runtime.rs      - CompiledFunction::call()

benches/vsp_jit.rs  - Performance benchmarks
examples/vsp_jit_poc.rs - Usage examples
```

## API Cheatsheet

### Compile Bytecode
```rust
use sil_core::vsp::jit::VspJit;

let mut jit = VspJit::new()?;
let func = jit.compile("name", &bytecode)?;
```

### Execute
```rust
let result: u64 = unsafe { func.call() };
```

### Supported Opcodes
- `0x01` - MOVI rd, imm24
- `0x10` - ADD ra, rb
- `0x11` - SUB ra, rb  
- `0x12` - MUL ra, rb
- `0x00` - HLT

## Bytecode Format

```
MOVI: [0x01, reg, imm_low, imm_mid, imm_high]
ADD:  [0x10, (rb << 4 | ra)]
SUB:  [0x11, (rb << 4 | ra)]
MUL:  [0x12, (rb << 4 | ra)]
HLT:  [0x00]
```

## Build Commands

```bash
# Build with JIT
cargo build --features jit

# Run tests
cargo test --features jit jit

# Run benchmarks
cargo bench --features jit vsp_jit

# Run example
cargo run --features jit --example vsp_jit_poc
```

## Adding New Opcodes

1. Update `translate_instruction_static()` in `compiler.rs`:
```rust
Opcode::YourOp => {
    let (ra, rb) = inst.reg_pair();
    let lhs = registers[ra].unwrap_or(...);
    let rhs = registers[rb].unwrap_or(...);
    let result = builder.ins().your_cranelift_op(lhs, rhs);
    registers[ra] = Some(result);
}
```

2. Add test in `compiler.rs`:
```rust
#[test]
fn test_jit_your_op() {
    let mut jit = VspJit::new().unwrap();
    let bytecode = vec![...];
    let func = jit.compile("test", &bytecode).unwrap();
    let result = unsafe { func.call() };
    assert_eq!(result, expected);
}
```

## Cranelift IR Reference

```rust
// Integer ops
builder.ins().iconst(types::I64, value)
builder.ins().iadd(lhs, rhs)
builder.ins().isub(lhs, rhs)
builder.ins().imul(lhs, rhs)
builder.ins().sdiv(lhs, rhs)
builder.ins().srem(lhs, rhs)

// Bitwise
builder.ins().band(lhs, rhs)
builder.ins().bor(lhs, rhs)
builder.ins().bxor(lhs, rhs)
builder.ins().ishl(val, shift)
builder.ins().sshr(val, shift)

// Control flow
builder.ins().jump(block)
builder.ins().brif(cond, then_block, else_block)
builder.ins().return_(&[val])

// Comparisons
builder.ins().icmp(IntCC::Equal, lhs, rhs)
```

## Error Handling

```rust
pub enum JitError {
    UnsupportedOpcode(Opcode),
    CraneliftError(String),
    InvalidBytecode,
}
```

## Performance Notes

- Compilation: ~100-200µs overhead
- Execution: 6-10µs per call (warm)
- Breakeven: ~30 calls
- Best for: Hot loops, repeated execution

## Known Limitations

- ❌ ARM64 (M1/M2/M3)
- ✅ x86_64 only
- No complex number ops yet
- No memory ops yet
- No control flow yet

## Debugging

```bash
# Enable Cranelift debug output
RUST_LOG=cranelift=debug cargo run --features jit

# Dump generated assembly
CRANELIFT_DISASM=1 cargo run --features jit

# Check compilation errors
cargo build --features jit 2>&1 | grep error
```

## Architecture Check

```bash
# Current arch
uname -m

# Required for JIT
x86_64 ✅
aarch64 ❌ (ARM64)
```

---

**Quick Start**: `cargo run --features jit --example vsp_jit_poc`
