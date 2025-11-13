//! # CPU State and Execution
//!
//! This module contains the CPU struct representing the 6502 processor state and
//! the fetch-decode-execute loop.
//!
//! ## CPU State
//!
//! The CPU maintains:
//! - **Registers**: Accumulator (A), index registers (X, Y)
//! - **Program counter** (PC): 16-bit address of next instruction
//! - **Stack pointer** (SP): 8-bit offset into stack page (0x0100-0x01FF)
//! - **Status flags**: N, V, B, D, I, Z, C (individual bool fields)
//! - **Cycle counter**: u64 monotonically increasing cycle count
//!
//! ## Execution Model
//!
//! The CPU executes instructions via:
//! - `step()`: Execute one instruction
//! - `run_for_cycles()`: Execute until cycle budget exhausted
//!
//! All opcodes return `UnimplementedOpcode` errors in this foundational feature.

use crate::{ExecutionError, MemoryBus, OPCODE_TABLE};

/// 6502 CPU state and execution context.
///
/// The CPU struct contains all processor state including registers, flags, program counter,
/// stack pointer, and cycle counter. It is generic over the memory implementation via the
/// `MemoryBus` trait.
///
/// # Type Parameters
///
/// * `M` - Memory bus implementation (must implement `MemoryBus` trait)
///
/// # Examples
///
/// ```
/// use cpu6502::{CPU, FlatMemory, MemoryBus};
///
/// // Create memory and set reset vector
/// let mut memory = FlatMemory::new();
/// memory.write(0xFFFC, 0x00); // Low byte
/// memory.write(0xFFFD, 0x80); // High byte (PC = 0x8000)
///
/// // Initialize CPU - loads PC from reset vector
/// let mut cpu = CPU::new(memory);
///
/// // Inspect initial state
/// assert_eq!(cpu.pc(), 0x8000);
/// assert_eq!(cpu.sp(), 0xFD);
/// assert_eq!(cpu.flag_i(), true); // Interrupt disable set on reset
/// assert_eq!(cpu.cycles(), 0);
/// ```
pub struct CPU<M: MemoryBus> {
    /// Accumulator register
    pub(crate) a: u8,

    /// X index register
    pub(crate) x: u8,

    /// Y index register
    pub(crate) y: u8,

    /// Program counter (address of next instruction)
    pub(crate) pc: u16,

    /// Stack pointer (0x0100 + sp gives full stack address)
    pub(crate) sp: u8,

    /// Negative flag (set if bit 7 of result is 1)
    pub(crate) flag_n: bool,

    /// Overflow flag (set on signed overflow)
    pub(crate) flag_v: bool,

    /// Break flag (set when BRK instruction executed)
    pub(crate) flag_b: bool,

    /// Decimal mode flag (enables BCD arithmetic)
    pub(crate) flag_d: bool,

    /// Interrupt disable flag (blocks IRQ when set)
    pub(crate) flag_i: bool,

    /// Zero flag (set if result is zero)
    pub(crate) flag_z: bool,

    /// Carry flag (set on unsigned overflow/underflow)
    pub(crate) flag_c: bool,

    /// Total CPU cycles executed
    pub(crate) cycles: u64,

    /// Memory bus implementation
    pub(crate) memory: M,
}

impl<M: MemoryBus> CPU<M> {
    /// Creates a new CPU with the given memory bus.
    ///
    /// The CPU is initialized to the 6502 power-on reset state:
    /// - Program counter (PC) is loaded from the reset vector at addresses 0xFFFC/0xFFFD (little-endian)
    /// - Stack pointer (SP) is set to 0xFD
    /// - Status register has Interrupt Disable flag set (I = true)
    /// - All other registers (A, X, Y) are zeroed
    /// - Cycle counter is reset to 0
    ///
    /// # Arguments
    ///
    /// * `memory` - A MemoryBus implementation that provides the reset vector
    ///
    /// # Examples
    ///
    /// ```
    /// use cpu6502::{CPU, FlatMemory, MemoryBus};
    ///
    /// let mut mem = FlatMemory::new();
    /// mem.write(0xFFFC, 0x00);
    /// mem.write(0xFFFD, 0x80);
    ///
    /// let cpu = CPU::new(mem);
    /// assert_eq!(cpu.pc(), 0x8000);
    /// ```
    pub fn new(memory: M) -> Self {
        // Read reset vector from 0xFFFC/0xFFFD (little-endian)
        let pc_low = memory.read(0xFFFC) as u16;
        let pc_high = memory.read(0xFFFD) as u16;
        let pc = (pc_high << 8) | pc_low;

        Self {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            pc,
            sp: 0xFD,
            flag_n: false,
            flag_v: false,
            flag_b: false,
            flag_d: false,
            flag_i: true, // Interrupt disable set on reset
            flag_z: false,
            flag_c: false,
            cycles: 0,
            memory,
        }
    }

    /// Executes one instruction and advances the CPU state.
    ///
    /// Performs the fetch-decode-execute cycle:
    /// 1. Fetch opcode byte at current PC
    /// 2. Look up instruction metadata in opcode table
    /// 3. Check if instruction is implemented
    /// 4. If not implemented, return error
    /// 5. Increment cycle counter by base cycles
    /// 6. Advance PC (would execute instruction in future features)
    ///
    /// Returns an error if the instruction is not yet implemented.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if instruction executed successfully (none in this feature)
    /// - `Err(ExecutionError::UnimplementedOpcode(opcode))` if instruction not implemented
    ///
    /// # Examples
    ///
    /// ```
    /// use cpu6502::{CPU, FlatMemory, MemoryBus, ExecutionError};
    ///
    /// let mut mem = FlatMemory::new();
    /// mem.write(0xFFFC, 0x00);
    /// mem.write(0xFFFD, 0x80);
    /// mem.write(0x8000, 0xEA); // NOP instruction
    ///
    /// let mut cpu = CPU::new(mem);
    ///
    /// match cpu.step() {
    ///     Ok(()) => println!("Instruction executed"),
    ///     Err(ExecutionError::UnimplementedOpcode(op)) => {
    ///         println!("Opcode 0x{:02X} not implemented", op);
    ///     }
    /// }
    /// ```
    pub fn step(&mut self) -> Result<(), ExecutionError> {
        // Fetch opcode at PC
        let opcode = self.memory.read(self.pc);

        // Decode: look up in opcode table
        let metadata = &OPCODE_TABLE[opcode as usize];

        // Check if implemented
        if !metadata.implemented {
            // Increment cycles even for unimplemented opcodes (for testing)
            self.cycles += metadata.base_cycles as u64;

            // Advance PC by instruction size (so we don't get stuck)
            self.pc = self.pc.wrapping_add(metadata.size_bytes as u16);

            return Err(ExecutionError::UnimplementedOpcode(opcode));
        }

        // Execute instruction (would implement actual instruction logic here in future)
        // For now, all instructions return unimplemented since implemented flag is always false

        // Increment cycle counter
        self.cycles += metadata.base_cycles as u64;

        // Advance PC
        self.pc = self.pc.wrapping_add(metadata.size_bytes as u16);

        Ok(())
    }

    /// Runs the CPU for a specified number of cycles.
    ///
    /// Executes instructions until the cycle budget is exhausted or an error occurs.
    /// Returns the actual number of cycles consumed (may be slightly more than budget
    /// due to instruction granularity).
    ///
    /// This is useful for frame-locked execution models where the CPU must run for
    /// an exact number of cycles per frame (e.g., 29780 cycles for 60Hz NTSC).
    ///
    /// # Arguments
    ///
    /// * `cycle_budget` - Maximum number of cycles to execute
    ///
    /// # Returns
    ///
    /// - `Ok(cycles_consumed)` if execution completed successfully
    /// - `Err(ExecutionError)` if an instruction failed
    ///
    /// # Examples
    ///
    /// ```
    /// use cpu6502::{CPU, FlatMemory, MemoryBus};
    ///
    /// let mut mem = FlatMemory::new();
    /// mem.write(0xFFFC, 0x00);
    /// mem.write(0xFFFD, 0x80);
    /// mem.write(0x8000, 0xEA); // NOP
    ///
    /// let mut cpu = CPU::new(mem);
    ///
    /// // Run CPU for one NTSC frame (60Hz, ~1.79 MHz)
    /// let cycles_per_frame = 29780;
    /// match cpu.run_for_cycles(cycles_per_frame) {
    ///     Ok(actual_cycles) => println!("Executed {} cycles", actual_cycles),
    ///     Err(e) => eprintln!("Execution error: {:?}", e),
    /// }
    /// ```
    pub fn run_for_cycles(&mut self, cycle_budget: u64) -> Result<u64, ExecutionError> {
        let start_cycles = self.cycles;
        let target_cycles = start_cycles + cycle_budget;

        while self.cycles < target_cycles {
            self.step()?;
        }

        Ok(self.cycles - start_cycles)
    }

    // ========== Register Getters ==========

    /// Returns the accumulator register value.
    ///
    /// # Examples
    ///
    /// ```
    /// use cpu6502::{CPU, FlatMemory, MemoryBus};
    ///
    /// let mut mem = FlatMemory::new();
    /// mem.write(0xFFFC, 0x00);
    /// mem.write(0xFFFD, 0x80);
    ///
    /// let cpu = CPU::new(mem);
    /// assert_eq!(cpu.a(), 0x00); // Initial value
    /// ```
    pub fn a(&self) -> u8 {
        self.a
    }

    /// Returns the X index register value.
    pub fn x(&self) -> u8 {
        self.x
    }

    /// Returns the Y index register value.
    pub fn y(&self) -> u8 {
        self.y
    }

    /// Returns the program counter value.
    pub fn pc(&self) -> u16 {
        self.pc
    }

    /// Returns the stack pointer value.
    ///
    /// Note: The full stack address is 0x0100 + SP. The stack grows downward from 0x01FF.
    pub fn sp(&self) -> u8 {
        self.sp
    }

    /// Returns the status register as a packed byte.
    ///
    /// Bit layout (NV-BDIZC):
    /// - Bit 7: N (Negative)
    /// - Bit 6: V (Overflow)
    /// - Bit 5: (unused, always 1)
    /// - Bit 4: B (Break)
    /// - Bit 3: D (Decimal)
    /// - Bit 2: I (Interrupt Disable)
    /// - Bit 1: Z (Zero)
    /// - Bit 0: C (Carry)
    ///
    /// # Examples
    ///
    /// ```
    /// use cpu6502::{CPU, FlatMemory, MemoryBus};
    ///
    /// let mut mem = FlatMemory::new();
    /// mem.write(0xFFFC, 0x00);
    /// mem.write(0xFFFD, 0x80);
    ///
    /// let cpu = CPU::new(mem);
    /// let status = cpu.status();
    ///
    /// // I flag set (bit 2), bit 5 always 1
    /// assert_eq!(status & 0b00100100, 0b00100100);
    /// ```
    pub fn status(&self) -> u8 {
        let mut status: u8 = 0b00100000; // Bit 5 always 1

        if self.flag_n {
            status |= 0b10000000;
        }
        if self.flag_v {
            status |= 0b01000000;
        }
        if self.flag_b {
            status |= 0b00010000;
        }
        if self.flag_d {
            status |= 0b00001000;
        }
        if self.flag_i {
            status |= 0b00000100;
        }
        if self.flag_z {
            status |= 0b00000010;
        }
        if self.flag_c {
            status |= 0b00000001;
        }

        status
    }

    /// Returns the total number of CPU cycles executed since initialization.
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    // ========== Status Flag Getters ==========

    /// Returns true if the Negative flag is set.
    pub fn flag_n(&self) -> bool {
        self.flag_n
    }

    /// Returns true if the Overflow flag is set.
    pub fn flag_v(&self) -> bool {
        self.flag_v
    }

    /// Returns true if the Break flag is set.
    pub fn flag_b(&self) -> bool {
        self.flag_b
    }

    /// Returns true if the Decimal mode flag is set.
    pub fn flag_d(&self) -> bool {
        self.flag_d
    }

    /// Returns true if the Interrupt Disable flag is set.
    pub fn flag_i(&self) -> bool {
        self.flag_i
    }

    /// Returns true if the Zero flag is set.
    pub fn flag_z(&self) -> bool {
        self.flag_z
    }

    /// Returns true if the Carry flag is set.
    pub fn flag_c(&self) -> bool {
        self.flag_c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlatMemory;

    #[test]
    fn test_cpu_initialization() {
        let mut mem = FlatMemory::new();

        // Set reset vector to 0x8000
        mem.write(0xFFFC, 0x00);
        mem.write(0xFFFD, 0x80);

        let cpu = CPU::new(mem);

        // Verify initial state
        assert_eq!(cpu.pc(), 0x8000);
        assert_eq!(cpu.sp(), 0xFD);
        assert_eq!(cpu.a(), 0x00);
        assert_eq!(cpu.x(), 0x00);
        assert_eq!(cpu.y(), 0x00);
        assert_eq!(cpu.cycles(), 0);

        // Verify status flags
        assert_eq!(cpu.flag_i(), true); // Interrupt disable set on reset
        assert_eq!(cpu.flag_n(), false);
        assert_eq!(cpu.flag_v(), false);
        assert_eq!(cpu.flag_b(), false);
        assert_eq!(cpu.flag_d(), false);
        assert_eq!(cpu.flag_z(), false);
        assert_eq!(cpu.flag_c(), false);
    }

    #[test]
    fn test_status_register_packing() {
        let mut mem = FlatMemory::new();
        mem.write(0xFFFC, 0x00);
        mem.write(0xFFFD, 0x80);

        let cpu = CPU::new(mem);
        let status = cpu.status();

        // Bit 5 always 1, I flag set (bit 2)
        assert_eq!(status & 0b00100000, 0b00100000); // Bit 5
        assert_eq!(status & 0b00000100, 0b00000100); // I flag
    }

    #[test]
    fn test_step_unimplemented() {
        let mut mem = FlatMemory::new();
        mem.write(0xFFFC, 0x00);
        mem.write(0xFFFD, 0x80);
        mem.write(0x8000, 0xEA); // NOP instruction (not implemented)

        let mut cpu = CPU::new(mem);
        let initial_cycles = cpu.cycles();

        match cpu.step() {
            Err(ExecutionError::UnimplementedOpcode(0xEA)) => {
                // Expected error
                assert!(cpu.cycles() > initial_cycles); // Cycles incremented
                assert_eq!(cpu.pc(), 0x8001); // PC advanced by instruction size
            }
            _ => panic!("Expected UnimplementedOpcode error"),
        }
    }

    #[test]
    fn test_run_for_cycles() {
        let mut mem = FlatMemory::new();
        mem.write(0xFFFC, 0x00);
        mem.write(0xFFFD, 0x80);

        // Fill memory with NOP instructions (0xEA, 2 cycles each)
        for addr in 0x8000..=0x8010 {
            mem.write(addr, 0xEA);
        }

        let mut cpu = CPU::new(mem);

        // Try to run for 10 cycles (should execute ~5 NOPs)
        let result = cpu.run_for_cycles(10);

        // Will fail with UnimplementedOpcode, but should have advanced
        assert!(result.is_err());
        assert!(cpu.cycles() >= 2); // At least one instruction executed
    }
}
