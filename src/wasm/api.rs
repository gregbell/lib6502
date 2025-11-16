//! WASM API for the 6502 emulator.
//!
//! Provides JavaScript-callable interfaces for CPU control, state inspection,
//! and assembly/disassembly operations.

use wasm_bindgen::prelude::*;
use crate::{CPU, FlatMemory, MemoryBus, assemble, disassemble, DisassemblyOptions};

/// JavaScript-compatible error wrapper
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsError {
    message: String,
}

#[wasm_bindgen]
impl JsError {
    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> JsError {
        JsError {
            message: message.to_string(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

/// Result of assembly operation
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct AssemblyResult {
    success: bool,
    machine_code: Vec<u8>,
    start_addr: u16,
    end_addr: u16,
    error_message: Option<String>,
    error_line: Option<usize>,
}

#[wasm_bindgen]
impl AssemblyResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn machine_code(&self) -> Vec<u8> {
        self.machine_code.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn start_addr(&self) -> u16 {
        self.start_addr
    }

    #[wasm_bindgen(getter)]
    pub fn end_addr(&self) -> u16 {
        self.end_addr
    }

    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error_line(&self) -> Option<usize> {
        self.error_line
    }
}

/// Result of disassembly operation
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct DisassemblyLine {
    address: u16,
    bytes: Vec<u8>,
    mnemonic: String,
    operand: String,
}

#[wasm_bindgen]
impl DisassemblyLine {
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> u16 {
        self.address
    }

    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn mnemonic(&self) -> String {
        self.mnemonic.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn operand(&self) -> String {
        self.operand.clone()
    }
}

/// Main emulator interface for JavaScript
#[wasm_bindgen]
pub struct Emulator6502 {
    cpu: CPU<FlatMemory>,
    program_start: u16,
    program_end: u16,
}

#[wasm_bindgen]
impl Emulator6502 {
    /// Create a new 6502 emulator instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize with flat 64KB memory
        let mut memory = FlatMemory::new();

        // Set reset vector to default program start (0x0600)
        memory.write(0xFFFC, 0x00);
        memory.write(0xFFFD, 0x06);

        let cpu = CPU::new(memory);

        Emulator6502 {
            cpu,
            program_start: 0x0600,
            program_end: 0x0600,
        }
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<(), JsError> {
        self.cpu.step()
            .map_err(|e| JsError::new(&format!("{:?}", e)))
    }

    /// Execute multiple cycles and return actual cycles executed
    pub fn run_for_cycles(&mut self, cycles: u32) -> Result<u32, JsError> {
        self.cpu.run_for_cycles(cycles as u64)
            .map(|c| c as u32)
            .map_err(|e| JsError::new(&format!("{:?}", e)))
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        // Create new memory and copy current memory state
        let mut new_memory = FlatMemory::new();

        // Copy all memory
        for addr in 0..=0xFFFF {
            let value = self.cpu.memory.read(addr);
            new_memory.write(addr, value);
        }

        // Create new CPU with the memory
        self.cpu = CPU::new(new_memory);
    }

    // Register getters
    #[wasm_bindgen(getter)]
    pub fn a(&self) -> u8 {
        self.cpu.a()
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> u8 {
        self.cpu.x()
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> u8 {
        self.cpu.y()
    }

    #[wasm_bindgen(getter)]
    pub fn pc(&self) -> u16 {
        self.cpu.pc()
    }

    #[wasm_bindgen(getter)]
    pub fn sp(&self) -> u8 {
        self.cpu.sp()
    }

    #[wasm_bindgen(getter)]
    pub fn cycles(&self) -> f64 {
        self.cpu.cycles() as f64  // Convert u64 to f64 for JavaScript
    }

    // Flag getters
    #[wasm_bindgen(getter)]
    pub fn flag_n(&self) -> bool {
        self.cpu.flag_n()
    }

    #[wasm_bindgen(getter)]
    pub fn flag_v(&self) -> bool {
        self.cpu.flag_v()
    }

    #[wasm_bindgen(getter)]
    pub fn flag_d(&self) -> bool {
        self.cpu.flag_d()
    }

    #[wasm_bindgen(getter)]
    pub fn flag_i(&self) -> bool {
        self.cpu.flag_i()
    }

    #[wasm_bindgen(getter)]
    pub fn flag_z(&self) -> bool {
        self.cpu.flag_z()
    }

    #[wasm_bindgen(getter)]
    pub fn flag_c(&self) -> bool {
        self.cpu.flag_c()
    }

    // Memory access methods

    /// Read a single byte from memory
    pub fn read_memory(&self, addr: u16) -> u8 {
        self.cpu.memory.read(addr)
    }

    /// Write a single byte to memory
    pub fn write_memory(&mut self, addr: u16, value: u8) {
        self.cpu.memory.write(addr, value);
    }

    /// Read a 256-byte page from memory (for efficient display)
    pub fn get_memory_page(&self, page: u8) -> Vec<u8> {
        let start = (page as u16) << 8;
        (0..256).map(|i| self.cpu.memory.read(start + i)).collect()
    }

    /// Load a program into memory and set PC
    pub fn load_program(&mut self, program: &[u8], start_addr: u16) {
        for (i, &byte) in program.iter().enumerate() {
            let addr = start_addr.wrapping_add(i as u16);
            self.cpu.memory.write(addr, byte);
        }
        self.cpu.set_pc(start_addr);
        self.program_start = start_addr;
        self.program_end = start_addr.wrapping_add(program.len() as u16);
    }

    /// Assemble 6502 assembly source code
    pub fn assemble(&self, source: String, start_addr: u16) -> AssemblyResult {
        match assemble(&source) {
            Ok(output) => {
                let end_addr = start_addr.wrapping_add(output.bytes.len() as u16);
                AssemblyResult {
                    success: true,
                    machine_code: output.bytes,
                    start_addr,
                    end_addr,
                    error_message: None,
                    error_line: None,
                }
            }
            Err(errors) => {
                // Return first error
                let first_error = &errors[0];
                AssemblyResult {
                    success: false,
                    machine_code: Vec::new(),
                    start_addr,
                    end_addr: start_addr,
                    error_message: Some(first_error.message.clone()),
                    error_line: Some(first_error.line),
                }
            }
        }
    }

    /// Assemble and load program in one step
    pub fn assemble_and_load(&mut self, source: String, start_addr: u16) -> AssemblyResult {
        let result = self.assemble(source, start_addr);
        if result.success {
            self.load_program(&result.machine_code, start_addr);
        }
        result
    }

    /// Disassemble memory starting at an address
    pub fn disassemble(&self, start_addr: u16, num_instructions: u32) -> Vec<JsValue> {
        let memory_vec: Vec<u8> = (0..=0xFFFF).map(|addr| self.cpu.memory.read(addr)).collect();

        let opts = DisassemblyOptions {
            start_address: start_addr,
            hex_dump: false,
            show_offsets: false,
        };

        let instructions = disassemble(&memory_vec, opts);

        // Take only the requested number of instructions
        instructions.iter().take(num_instructions as usize).map(|instr| {
            let mut bytes = vec![instr.opcode];
            bytes.extend_from_slice(&instr.operand_bytes);

            let line = DisassemblyLine {
                address: instr.address,
                bytes,
                mnemonic: instr.mnemonic.to_string(),
                operand: instr.operand_bytes.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" "),
            };
            JsValue::from(line)
        }).collect()
    }

    /// Get the program start address
    #[wasm_bindgen(getter)]
    pub fn program_start(&self) -> u16 {
        self.program_start
    }

    /// Get the program end address
    #[wasm_bindgen(getter)]
    pub fn program_end(&self) -> u16 {
        self.program_end
    }
}

impl Default for Emulator6502 {
    fn default() -> Self {
        Self::new()
    }
}
