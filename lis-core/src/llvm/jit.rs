//! JIT Execution Engine for LIS
//!
//! Provides just-in-time compilation and execution using LLVM's MCJIT.

use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::OptimizationLevel;

use crate::error::{Error, Result};

/// Type alias for JIT-compiled main function
pub type MainFn = unsafe extern "C" fn() -> i64;

/// JIT Execution Engine
pub struct JitEngine<'ctx> {
    engine: ExecutionEngine<'ctx>,
}

impl<'ctx> JitEngine<'ctx> {
    /// Create a new JIT engine from an LLVM module
    pub fn new(module: Module<'ctx>) -> Result<Self> {
        let engine = module
            .create_jit_execution_engine(OptimizationLevel::Aggressive)
            .map_err(|e| Error::CodeGenError { message: e.to_string() })?;

        Ok(Self { engine })
    }

    /// Get a JIT-compiled function by name
    pub fn get_function<F>(&self, name: &str) -> Result<JitFunction<'ctx, F>>
    where
        F: inkwell::execution_engine::UnsafeFunctionPointer,
    {
        unsafe {
            self.engine.get_function(name).map_err(|e| Error::CodeGenError {
                message: format!("Function '{}' not found: {}", name, e),
            })
        }
    }

    /// Run the main function and return its result
    pub fn run_main(&self) -> Result<i64> {
        let main_fn: JitFunction<MainFn> = self.get_function("main")?;
        Ok(unsafe { main_fn.call() })
    }

    /// Run a named function with no arguments
    pub fn run_function(&self, name: &str) -> Result<i64> {
        let func: JitFunction<MainFn> = self.get_function(name)?;
        Ok(unsafe { func.call() })
    }

    /// Get raw function pointer for a named function
    pub fn get_function_address(&self, name: &str) -> Result<u64> {
        self.engine
            .get_function_address(name)
            .map_err(|e| Error::CodeGenError {
                message: format!("Function '{}' address not found: {}", name, e),
            })
    }
}

/// Wrapper for JIT-compiled function pointer
pub struct JitFunctionPtr {
    address: u64,
}

impl JitFunctionPtr {
    /// Create from address
    pub fn new(address: u64) -> Self {
        Self { address }
    }

    /// Get the raw address
    pub fn address(&self) -> u64 {
        self.address
    }

    /// Call as a function returning i64
    pub unsafe fn call_i64(&self) -> i64 {
        let func: extern "C" fn() -> i64 = std::mem::transmute(self.address);
        func()
    }

    /// Call as a function taking i64 and returning i64
    pub unsafe fn call_i64_i64(&self, arg: i64) -> i64 {
        let func: extern "C" fn(i64) -> i64 = std::mem::transmute(self.address);
        func(arg)
    }

    /// Call as a function taking two i64s and returning i64
    pub unsafe fn call_i64_i64_i64(&self, arg1: i64, arg2: i64) -> i64 {
        let func: extern "C" fn(i64, i64) -> i64 = std::mem::transmute(self.address);
        func(arg1, arg2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llvm::LlvmCodegen;
    use inkwell::context::Context;

    #[test]
    fn test_jit_simple_return() {
        let source = r#"
            fn main() {
                return 42;
            }
        "#;

        let tokens = crate::Lexer::new(source).tokenize().unwrap();
        let ast = crate::Parser::new(tokens).parse().unwrap();

        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test");
        let module = codegen.compile(&ast).unwrap();

        let engine = JitEngine::new(module).unwrap();
        let result = engine.run_main().unwrap();

        assert_eq!(result, 42);
    }

    #[test]
    fn test_jit_arithmetic() {
        let source = r#"
            fn main() {
                let a = 100;
                let b = 23;
                let c = a - b;
                return c * 2;
            }
        "#;

        let tokens = crate::Lexer::new(source).tokenize().unwrap();
        let ast = crate::Parser::new(tokens).parse().unwrap();

        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test");
        let module = codegen.compile(&ast).unwrap();

        let engine = JitEngine::new(module).unwrap();
        let result = engine.run_main().unwrap();

        assert_eq!(result, 154); // (100 - 23) * 2 = 154
    }

    #[test]
    fn test_jit_conditionals() {
        let source = r#"
            fn main() {
                let x = 10;
                if x > 5 {
                    return 1;
                } else {
                    return 0;
                }
            }
        "#;

        let tokens = crate::Lexer::new(source).tokenize().unwrap();
        let ast = crate::Parser::new(tokens).parse().unwrap();

        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test");
        let module = codegen.compile(&ast).unwrap();

        let engine = JitEngine::new(module).unwrap();
        let result = engine.run_main().unwrap();

        assert_eq!(result, 1);
    }

    #[test]
    fn test_jit_loop_with_break() {
        let source = r#"
            fn main() {
                let i = 0;
                let sum = 0;
                loop {
                    if i >= 5 {
                        break;
                    }
                    sum = sum + i;
                    i = i + 1;
                }
                return sum;
            }
        "#;

        let tokens = crate::Lexer::new(source).tokenize().unwrap();
        let ast = crate::Parser::new(tokens).parse().unwrap();

        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test");
        let module = codegen.compile(&ast).unwrap();

        let engine = JitEngine::new(module).unwrap();
        let result = engine.run_main().unwrap();

        assert_eq!(result, 10); // 0 + 1 + 2 + 3 + 4 = 10
    }

    #[test]
    fn test_jit_function_call() {
        let source = r#"
            fn add(a, b) {
                return a + b;
            }

            fn main() {
                let x = add(10, 32);
                return x;
            }
        "#;

        let tokens = crate::Lexer::new(source).tokenize().unwrap();
        let ast = crate::Parser::new(tokens).parse().unwrap();

        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test");
        let module = codegen.compile(&ast).unwrap();

        let engine = JitEngine::new(module).unwrap();
        let result = engine.run_main().unwrap();

        assert_eq!(result, 42);
    }

    #[test]
    fn test_jit_nested_calls() {
        let source = r#"
            fn double(x) {
                return x * 2;
            }

            fn quadruple(x) {
                return double(double(x));
            }

            fn main() {
                return quadruple(10);
            }
        "#;

        let tokens = crate::Lexer::new(source).tokenize().unwrap();
        let ast = crate::Parser::new(tokens).parse().unwrap();

        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test");
        let module = codegen.compile(&ast).unwrap();

        let engine = JitEngine::new(module).unwrap();
        let result = engine.run_main().unwrap();

        assert_eq!(result, 40); // 10 * 2 * 2 = 40
    }

    #[test]
    fn test_jit_fibonacci() {
        let source = r#"
            fn fib(n) {
                if n <= 1 {
                    return n;
                }
                return fib(n - 1) + fib(n - 2);
            }

            fn main() {
                return fib(10);
            }
        "#;

        let tokens = crate::Lexer::new(source).tokenize().unwrap();
        let ast = crate::Parser::new(tokens).parse().unwrap();

        let context = Context::create();
        let codegen = LlvmCodegen::new(&context, "test");
        let module = codegen.compile(&ast).unwrap();

        let engine = JitEngine::new(module).unwrap();
        let result = engine.run_main().unwrap();

        assert_eq!(result, 55); // fib(10) = 55
    }
}
