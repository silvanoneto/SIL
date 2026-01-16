//! Executor for VSP bytecode

use crate::error::{RuntimeError, RuntimeResult};
use sil_core::prelude::*;

/// Executor simples de VSP assembly
///
/// Por enquanto, este é um intérprete simplificado que executa
/// diretamente o assembly VSP sem passar por bytecode.
pub struct Executor {
    state: SilState,
    registers: [ByteSil; 16],
    pc: usize, // Program counter
}

impl Executor {
    /// Cria novo executor
    pub fn new() -> Self {
        Self {
            state: SilState::neutral(),
            registers: [ByteSil::default(); 16],
            pc: 0,
        }
    }

    /// Executa programa VSP assembly
    pub fn execute(&mut self, assembly: &str) -> RuntimeResult<SilState> {
        let lines: Vec<&str> = assembly
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('.') && !l.starts_with(';'))
            .collect();

        self.pc = 0;

        while self.pc < lines.len() {
            let line = lines[self.pc];

            // Ignorar labels
            if line.ends_with(':') {
                self.pc += 1;
                continue;
            }

            // Executar instrução
            self.execute_instruction(line)?;
            self.pc += 1;
        }

        Ok(self.state)
    }

    /// Executa uma instrução individual
    fn execute_instruction(&mut self, instruction: &str) -> RuntimeResult<()> {
        let parts: Vec<&str> = instruction.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let opcode = parts[0];

        match opcode {
            "NOP" => { /* No operation */ }

            "MOVI" => {
                // MOVI R0, 42
                if parts.len() < 3 {
                    return Err(RuntimeError::ExecutionError(
                        format!("MOVI requires 2 operands: {}", instruction)
                    ));
                }
                let reg = self.parse_register(parts[1])?;
                let value = self.parse_immediate(parts[2])?;
                self.registers[reg as usize] = ByteSil::new(value, 0);
            }

            "MOV" => {
                // MOV R0, R1
                if parts.len() < 3 {
                    return Err(RuntimeError::ExecutionError(
                        format!("MOV requires 2 operands: {}", instruction)
                    ));
                }
                let dest = self.parse_register(parts[1])?;
                let src = self.parse_register(parts[2])?;
                self.registers[dest as usize] = self.registers[src as usize];
            }

            "ADD" => {
                // ADD R0, R1, R2  (R0 = R1 + R2)
                if parts.len() < 4 {
                    return Err(RuntimeError::ExecutionError(
                        format!("ADD requires 3 operands: {}", instruction)
                    ));
                }
                let dest = self.parse_register(parts[1])?;
                let src1 = self.parse_register(parts[2])?;
                let src2 = self.parse_register(parts[3])?;

                let val1 = self.registers[src1 as usize];
                let val2 = self.registers[src2 as usize];

                self.registers[dest as usize] = ByteSil::new(
                    (val1.rho + val2.rho).clamp(-8, 7),
                    val1.theta.wrapping_add(val2.theta)
                );
            }

            "SUB" => {
                // SUB R0, R1, R2  (R0 = R1 - R2)
                if parts.len() < 4 {
                    return Err(RuntimeError::ExecutionError(
                        format!("SUB requires 3 operands: {}", instruction)
                    ));
                }
                let dest = self.parse_register(parts[1])?;
                let src1 = self.parse_register(parts[2])?;
                let src2 = self.parse_register(parts[3])?;

                let val1 = self.registers[src1 as usize];
                let val2 = self.registers[src2 as usize];

                self.registers[dest as usize] = ByteSil::new(
                    (val1.rho - val2.rho).clamp(-8, 7),
                    val1.theta.wrapping_sub(val2.theta)
                );
            }

            "MUL" => {
                // MUL R0, R1, R2  (R0 = R1 * R2)
                if parts.len() < 4 {
                    return Err(RuntimeError::ExecutionError(
                        format!("MUL requires 3 operands: {}", instruction)
                    ));
                }
                let dest = self.parse_register(parts[1])?;
                let src1 = self.parse_register(parts[2])?;
                let src2 = self.parse_register(parts[3])?;

                let val1 = self.registers[src1 as usize];
                let val2 = self.registers[src2 as usize];

                // Multiplicação complexa (log-polar)
                self.registers[dest as usize] = ByteSil::new(
                    (val1.rho + val2.rho).clamp(-8, 7),
                    val1.theta.wrapping_add(val2.theta)
                );
            }

            "LOADL" => {
                // LOADL R0, R1, L0  (Load layer L0 from state in R1 into R0)
                if parts.len() < 4 {
                    return Err(RuntimeError::ExecutionError(
                        format!("LOADL requires 3 operands: {}", instruction)
                    ));
                }
                let dest = self.parse_register(parts[1])?;
                let layer = self.parse_layer(parts[3])?;

                self.registers[dest as usize] = self.state.get(layer as usize);
            }

            "STOREL" => {
                // STOREL R0, L0, R1  (Store R1 into layer L0 of state in R0)
                if parts.len() < 4 {
                    return Err(RuntimeError::ExecutionError(
                        format!("STOREL requires 3 operands: {}", instruction)
                    ));
                }
                let layer = self.parse_layer(parts[2])?;
                let src = self.parse_register(parts[3])?;

                self.state = self.state.with_layer(layer as usize, self.registers[src as usize]);
            }

            "HLT" => {
                // Halt execution
                return Ok(());
            }

            "RET" => {
                // Return (for now, same as HLT in this simple executor)
                return Ok(());
            }

            _ => {
                // Instrução desconhecida - ignorar por enquanto
                // (JMP, CALL, etc. precisariam de label resolution)
            }
        }

        Ok(())
    }

    /// Parse register name (R0-RF)
    fn parse_register(&self, reg: &str) -> RuntimeResult<u8> {
        let reg = reg.trim_end_matches(',');

        if !reg.starts_with('R') {
            return Err(RuntimeError::ExecutionError(
                format!("Invalid register: {}", reg)
            ));
        }

        let num_str = &reg[1..];
        u8::from_str_radix(num_str, 16)
            .map_err(|_| RuntimeError::ExecutionError(
                format!("Invalid register number: {}", reg)
            ))
    }

    /// Parse immediate value
    fn parse_immediate(&self, imm: &str) -> RuntimeResult<i8> {
        let imm = imm.trim_end_matches(',');

        imm.parse::<i8>()
            .map_err(|_| RuntimeError::ExecutionError(
                format!("Invalid immediate value: {}", imm)
            ))
    }

    /// Parse layer (L0-LF)
    fn parse_layer(&self, layer: &str) -> RuntimeResult<u8> {
        let layer = layer.trim_end_matches(',');

        if !layer.starts_with('L') {
            return Err(RuntimeError::ExecutionError(
                format!("Invalid layer: {}", layer)
            ));
        }

        let num_str = &layer[1..];
        u8::from_str_radix(num_str, 16)
            .map_err(|_| RuntimeError::ExecutionError(
                format!("Invalid layer number: {}", layer)
            ))
    }

    /// Retorna estado atual
    pub fn state(&self) -> SilState {
        self.state
    }

    /// Retorna registradores
    pub fn registers(&self) -> &[ByteSil; 16] {
        &self.registers
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_movi() {
        let mut executor = Executor::new();
        executor.execute_instruction("MOVI R0, 5").unwrap();
        assert_eq!(executor.registers[0].rho, 5);
    }

    #[test]
    fn test_executor_add() {
        let mut executor = Executor::new();
        executor.execute_instruction("MOVI R1, 3").unwrap();
        executor.execute_instruction("MOVI R2, 4").unwrap();
        executor.execute_instruction("ADD R0, R1, R2").unwrap();
        assert_eq!(executor.registers[0].rho, 7);
    }

    #[test]
    fn test_executor_simple_program() {
        let program = r#"
            .mode SIL-128

            .code
            main:
                MOVI R0, 3
                MOVI R1, 4
                ADD R2, R0, R1
                STOREL R2, L0, R2
                HLT
        "#;

        let mut executor = Executor::new();
        let result = executor.execute(program).unwrap();

        // 3 + 4 = 7, que é o valor máximo de rho em ByteSil
        assert_eq!(result.get(0).rho, 7);
    }
}
