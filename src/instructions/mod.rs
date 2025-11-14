//! # 6502 Instruction Implementations
//!
//! This module contains the implementations of all 6502 instructions, organized by category.
//! Each instruction is implemented as a standalone function that takes a mutable reference
//! to the CPU and the opcode byte.
//!
//! ## Categories
//!
//! - **alu**: Arithmetic and logic operations (ADC, SBC, AND, ORA, EOR, CMP, CPX, CPY, BIT)
//! - **branches**: Conditional branch instructions (BCC, BCS, BEQ, BNE, BMI, BPL, BVC, BVS)
//! - **shifts**: Shift and rotate operations (ASL, LSR, ROL, ROR)
//! - **load_store**: Load and store instructions (LDA, LDX, LDY, STA, STX, STY)
//! - **inc_dec**: Increment and decrement operations (INC, DEC, INX, INY, DEX, DEY)
//! - **control**: Control flow instructions (JMP, JSR, RTS, RTI, BRK, NOP)
//! - **stack**: Stack operations (PHA, PHP, PLA, PLP)
//! - **flags**: Status flag manipulation (CLC, SEC, CLI, SEI, CLD, SED, CLV)
//! - **transfer**: Register transfer operations (TAX, TAY, TXA, TYA, TSX, TXS)

pub mod alu;
pub mod branches;
pub mod control;
pub mod flags;
pub mod inc_dec;
pub mod shifts;

// Future modules (to be implemented):
// pub mod load_store;
// pub mod stack;
// pub mod transfer;
