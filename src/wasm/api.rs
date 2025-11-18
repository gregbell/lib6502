//! WASM API for the 6502 emulator.
//!
//! Provides JavaScript-callable interfaces for CPU control, state inspection,
//! and assembly/disassembly operations.

use crate::{
    assemble, disassemble, DisassemblyOptions, MappedMemory, RamDevice, RomDevice,
    Uart6551, MemoryBus, Device, CPU,
};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Shared UART wrapper that implements Device
/// Allows the UART to be used both in MappedMemory and accessed separately for receive_byte()
struct SharedUart {
    uart: Rc<RefCell<Uart6551>>,
}

impl SharedUart {
    fn new(uart: Rc<RefCell<Uart6551>>) -> Self {
        SharedUart { uart }
    }
}

impl Device for SharedUart {
    fn read(&self, offset: u16) -> u8 {
        self.uart.borrow().read(offset)
    }

    fn write(&mut self, offset: u16, value: u8) {
        self.uart.borrow_mut().write(offset, value);
    }

    fn size(&self) -> u16 {
        self.uart.borrow().size()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

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
    cpu: CPU<MappedMemory>,
    uart: Rc<RefCell<Uart6551>>,
    on_transmit: js_sys::Function,
    program_start: u16,
    program_end: u16,
}

#[wasm_bindgen]
impl Emulator6502 {
    /// Create a new 6502 emulator instance with UART support
    #[wasm_bindgen(constructor)]
    pub fn new(on_transmit: js_sys::Function) -> Self {
        // Initialize MappedMemory with device mapping
        let mut memory = MappedMemory::new();

        // Add RAM at $0000-$7FFF (32KB)
        memory
            .add_device(0x0000, Box::new(RamDevice::new(32768)))
            .expect("Failed to add RAM device");

        // Create UART device with transmit callback
        let uart = Rc::new(RefCell::new(Uart6551::new()));
        let on_transmit_clone = on_transmit.clone();
        uart.borrow_mut().set_transmit_callback(move |byte| {
            let char_str = String::from_utf8(vec![byte]).unwrap_or_else(|_| "?".to_string());
            let _ = on_transmit_clone.call1(&JsValue::NULL, &JsValue::from_str(&char_str));
        });

        // Add UART at $A000-$A003 using SharedUart wrapper
        memory
            .add_device(0xA000, Box::new(SharedUart::new(Rc::clone(&uart))))
            .expect("Failed to add UART device");

        // Create ROM with reset vector pointing to $0600
        let mut rom_data = vec![0xEA; 16384]; // 16KB of NOP instructions
        rom_data[0x3FFC] = 0x00; // Reset vector low byte ($FFFC in ROM = offset $3FFC)
        rom_data[0x3FFD] = 0x06; // Reset vector high byte ($FFFD in ROM = offset $3FFD)

        // Add ROM at $C000-$FFFF (16KB including vectors)
        memory
            .add_device(0xC000, Box::new(RomDevice::new(rom_data)))
            .expect("Failed to add ROM device");

        let cpu = CPU::new(memory);

        Emulator6502 {
            cpu,
            uart,
            on_transmit,
            program_start: 0x0600,
            program_end: 0x0600,
        }
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<(), JsError> {
        self.cpu
            .step()
            .map_err(|e| JsError::new(&format!("{:?}", e)))
    }

    /// Execute multiple cycles and return actual cycles executed
    pub fn run_for_cycles(&mut self, cycles: u32) -> Result<u32, JsError> {
        self.cpu
            .run_for_cycles(cycles as u64)
            .map(|c| c as u32)
            .map_err(|e| JsError::new(&format!("{:?}", e)))
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        // Save current RAM contents
        let mut ram_backup = Vec::with_capacity(32768);
        for addr in 0x0000..=0x7FFF {
            ram_backup.push(self.cpu.memory.read(addr));
        }

        // Create new MappedMemory
        let mut memory = MappedMemory::new();

        // Restore RAM at $0000-$7FFF (32KB)
        let mut ram = RamDevice::new(32768);
        for (offset, &byte) in ram_backup.iter().enumerate() {
            ram.write(offset as u16, byte);
        }
        memory
            .add_device(0x0000, Box::new(ram))
            .expect("Failed to add RAM device");

        // Create fresh UART with transmit callback
        let uart = Rc::new(RefCell::new(Uart6551::new()));
        let on_transmit_clone = self.on_transmit.clone();
        uart.borrow_mut().set_transmit_callback(move |byte| {
            let char_str = String::from_utf8(vec![byte]).unwrap_or_else(|_| "?".to_string());
            let _ = on_transmit_clone.call1(&JsValue::NULL, &JsValue::from_str(&char_str));
        });

        // Add UART at $A000-$A003 using SharedUart wrapper
        memory
            .add_device(0xA000, Box::new(SharedUart::new(Rc::clone(&uart))))
            .expect("Failed to add UART device");

        // Create ROM with reset vector pointing to $0600
        let mut rom_data = vec![0xEA; 16384]; // 16KB of NOP instructions
        rom_data[0x3FFC] = 0x00; // Reset vector low byte
        rom_data[0x3FFD] = 0x06; // Reset vector high byte
        memory
            .add_device(0xC000, Box::new(RomDevice::new(rom_data)))
            .expect("Failed to add ROM device");

        // Create new CPU with reset memory
        self.cpu = CPU::new(memory);
        self.uart = uart;
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
        self.cpu.cycles() as f64 // Convert u64 to f64 for JavaScript
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

    // Register setters

    /// Set the program counter
    pub fn set_pc(&mut self, addr: u16) {
        self.cpu.set_pc(addr);
    }

    // UART methods

    /// Receive a character from the terminal into the UART buffer
    pub fn receive_char(&mut self, byte: u8) {
        self.uart.borrow_mut().receive_byte(byte);
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
        let memory_vec: Vec<u8> = (0..=0xFFFF)
            .map(|addr| self.cpu.memory.read(addr))
            .collect();

        let opts = DisassemblyOptions {
            start_address: start_addr,
            hex_dump: false,
            show_offsets: false,
        };

        let instructions = disassemble(&memory_vec, opts);

        // Take only the requested number of instructions
        instructions
            .iter()
            .take(num_instructions as usize)
            .map(|instr| {
                let mut bytes = vec![instr.opcode];
                bytes.extend_from_slice(&instr.operand_bytes);

                let line = DisassemblyLine {
                    address: instr.address,
                    bytes,
                    mnemonic: instr.mnemonic.to_string(),
                    operand: instr
                        .operand_bytes
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                };
                JsValue::from(line)
            })
            .collect()
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
