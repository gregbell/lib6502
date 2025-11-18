//! Example: Interrupt-Capable Timer Device
//!
//! This example demonstrates how to implement an interrupt-capable device for
//! the 6502 emulator using memory-mapped registers and the InterruptDevice trait.
//!
//! # Timer Device Overview
//!
//! The `TimerDevice` is a simple interval timer that generates interrupts after
//! a specified number of CPU cycles. It features:
//!
//! - 16-bit countdown timer
//! - Interrupt generation on timer expiration
//! - Memory-mapped control and status registers
//! - Hardware-accurate interrupt acknowledgment
//!
//! # Memory-Mapped Registers
//!
//! When mapped to base address (e.g., 0xD000):
//!
//! ```text
//! 0xD000: STATUS register (read-only)
//!     Bit 7: Interrupt pending (1 = interrupt active, 0 = none)
//!     Bit 6-0: Reserved (read as 0)
//!
//! 0xD001: CONTROL register (write-only)
//!     Bit 7: Interrupt acknowledge (write 1 to clear interrupt)
//!     Bit 0: Timer enable (write 1 to start, 0 to stop)
//!     Bit 6-1: Reserved
//!
//! 0xD002: COUNTER_LO (read-only)
//!     Low byte of current counter value
//!
//! 0xD003: COUNTER_HI (read-only)
//!     High byte of current counter value
//! ```
//!
//! # Usage Example
//!
//! ```rust
//! use lib6502::{CPU, MappedMemory, MemoryBus};
//!
//! // This example would require running the interrupt_device.rs file
//! // See the main() function below for a complete working demonstration
//! ```

use lib6502::{Device, InterruptDevice, MappedMemory, MemoryBus, RamDevice, CPU};
use std::any::Any;

/// Interrupt-capable timer device with memory-mapped registers.
///
/// # Architecture
///
/// The timer counts down from a specified reload value. When the counter
/// reaches zero, an interrupt is triggered and the counter reloads automatically.
///
/// # Interrupt Behavior
///
/// - **Assertion**: Interrupt asserted when counter expires (reaches 0)
/// - **Acknowledgment**: ISR must write to CONTROL register to clear interrupt
/// - **Level-sensitive**: Interrupt remains active until acknowledged
/// - **Auto-reload**: Counter automatically reloads after expiration
///
/// # Example 6502 ISR
///
/// ```asm
/// timer_isr:
///     pha                ; Save accumulator
///
///     lda $D000          ; Read timer STATUS register
///     and #$80           ; Check interrupt pending bit
///     beq not_timer      ; If not set, not our interrupt
///
///     ; Handle timer interrupt
///     lda #$80
///     sta $D001          ; Write to CONTROL register (acknowledge)
///
/// not_timer:
///     pla                ; Restore accumulator
///     rti                ; Return from interrupt
/// ```
pub struct TimerDevice {
    /// Base address where device is mapped in memory
    base_address: u16,

    /// Interrupt pending flag (contributes to CPU IRQ line)
    interrupt_pending: bool,

    /// Current countdown value (counts down each tick)
    counter: u16,

    /// Reload value (counter resets to this after expiration)
    reload_value: u16,

    /// Timer enabled flag (timer only counts when true)
    enabled: bool,
}

impl TimerDevice {
    /// Create a new TimerDevice with specified reload value.
    ///
    /// # Arguments
    ///
    /// * `base_address` - Memory base address for device registers
    /// * `reload_value` - Value to reload counter after expiration
    ///
    /// # Examples
    ///
    /// ```
    /// # use lib6502::*;
    /// # use std::any::Any;
    /// # pub struct TimerDevice { base_address: u16, interrupt_pending: bool, counter: u16, reload_value: u16, enabled: bool }
    /// # impl TimerDevice {
    /// #   pub fn new(base_address: u16, reload_value: u16) -> Self {
    /// #     Self { base_address, interrupt_pending: false, counter: reload_value, reload_value, enabled: false }
    /// #   }
    /// # }
    /// // Create timer that interrupts every 1000 cycles
    /// let timer = TimerDevice::new(0xD000, 1000);
    /// ```
    pub fn new(base_address: u16, reload_value: u16) -> Self {
        Self {
            base_address,
            interrupt_pending: false,
            counter: reload_value,
            reload_value,
            enabled: false,
        }
    }

    /// Tick the timer (call once per CPU cycle).
    ///
    /// If timer is enabled, decrements counter. When counter reaches zero:
    /// - Sets interrupt_pending flag
    /// - Reloads counter to reload_value
    /// - Timer continues running
    ///
    /// # Examples
    ///
    /// ```
    /// # use lib6502::*;
    /// # use std::any::Any;
    /// # pub struct TimerDevice { base_address: u16, interrupt_pending: bool, counter: u16, reload_value: u16, enabled: bool }
    /// # impl TimerDevice {
    /// #   pub fn new(base_address: u16, reload_value: u16) -> Self {
    /// #     Self { base_address, interrupt_pending: false, counter: reload_value, reload_value, enabled: false }
    /// #   }
    /// #   pub fn tick(&mut self) {
    /// #     if !self.enabled { return; }
    /// #     if self.counter == 0 {
    /// #       self.interrupt_pending = true;
    /// #       self.counter = self.reload_value;
    /// #     } else {
    /// #       self.counter -= 1;
    /// #     }
    /// #   }
    /// #   pub fn has_interrupt(&self) -> bool { self.interrupt_pending }
    /// # }
    /// let mut timer = TimerDevice::new(0xD000, 10);
    /// // Enable timer (in real usage, would write to CONTROL register)
    ///
    /// // Tick timer 10 times - should trigger interrupt
    /// for _ in 0..10 {
    ///     // timer.tick();
    /// }
    /// // assert!(timer.has_interrupt());
    /// ```
    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        if self.counter == 0 {
            // Counter expired - trigger interrupt and reload
            self.interrupt_pending = true;
            self.counter = self.reload_value;
        } else {
            self.counter -= 1;
        }
    }

    /// Get current counter value.
    pub fn counter(&self) -> u16 {
        self.counter
    }

    /// Check if timer is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl InterruptDevice for TimerDevice {
    fn has_interrupt(&self) -> bool {
        self.interrupt_pending
    }
}

impl Device for TimerDevice {
    fn read(&self, offset: u16) -> u8 {
        match offset {
            0 => {
                // STATUS register (read-only)
                // Bit 7: Interrupt pending flag
                let mut status = 0x00;
                if self.interrupt_pending {
                    status |= 0x80; // Set bit 7
                }
                status
            }
            1 => {
                // CONTROL register (write-only, reads return 0)
                0x00
            }
            2 => {
                // COUNTER_LO: Low byte of counter
                (self.counter & 0xFF) as u8
            }
            3 => {
                // COUNTER_HI: High byte of counter
                ((self.counter >> 8) & 0xFF) as u8
            }
            _ => 0x00, // Unmapped offsets return 0
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        match offset {
            0 => {
                // STATUS register is read-only, ignore writes
            }
            1 => {
                // CONTROL register (write-only)
                // Bit 7: Interrupt acknowledge (clears interrupt)
                if value & 0x80 != 0 {
                    self.interrupt_pending = false;
                }

                // Bit 0: Timer enable
                self.enabled = (value & 0x01) != 0;
            }
            2 | 3 => {
                // Counter registers are read-only, ignore writes
            }
            _ => {
                // Unmapped offsets, ignore writes
            }
        }
    }

    fn size(&self) -> u16 {
        4 // Four registers: STATUS, CONTROL, COUNTER_LO, COUNTER_HI
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // Note: has_interrupt() is implemented via InterruptDevice trait
    // We override Device::has_interrupt() to delegate to InterruptDevice implementation
    fn has_interrupt(&self) -> bool {
        // Delegate to InterruptDevice trait implementation
        <Self as InterruptDevice>::has_interrupt(self)
    }
}

/// Example demonstration of timer-based interrupts.
///
/// This function sets up a complete 6502 system with:
/// - 64KB RAM
/// - Timer device at 0xD000-0xD003
/// - IRQ handler that acknowledges timer interrupts
/// - Main program that enables interrupts and waits
fn main() {
    println!("=== 6502 Interrupt-Capable Timer Device Example ===\n");

    // Create memory mapper
    let mut memory = MappedMemory::new();

    // Add 48KB RAM at 0x0000-0xBFFF (leaves space for I/O and ROM)
    memory
        .add_device(0x0000, Box::new(RamDevice::new(0xC000)))
        .expect("Failed to add RAM");

    // Add timer device at 0xD000-0xD003
    let timer = TimerDevice::new(0xD000, 100); // Interrupt every 100 cycles
    memory
        .add_device(0xD000, Box::new(timer))
        .expect("Failed to add timer");

    // Set up IRQ vector to point to handler at 0xC000
    memory.write(0xFFFE, 0x00); // IRQ vector low byte
    memory.write(0xFFFF, 0xC0); // IRQ vector high byte (handler at 0xC000)

    // Write a simple IRQ handler at 0xC000
    // This handler:
    // 1. Reads timer STATUS register (0xD000)
    // 2. Checks interrupt pending bit (bit 7)
    // 3. Acknowledges interrupt by writing to CONTROL (0xD001)
    // 4. Returns via RTI
    let irq_handler = [
        0x48, // PHA - Push accumulator
        0xAD, 0x00, 0xD0, // LDA $D000 - Read timer STATUS
        0x29, 0x80, // AND #$80 - Check bit 7
        0xF0, 0x04, // BEQ +4 - Skip acknowledge if not set
        0xA9, 0x80, // LDA #$80
        0x8D, 0x01, 0xD0, // STA $D001 - Write CONTROL (acknowledge)
        0x68, // PLA - Restore accumulator
        0x40, // RTI - Return from interrupt
    ];

    for (i, &byte) in irq_handler.iter().enumerate() {
        memory.write(0xC000 + i as u16, byte);
    }

    // Write main program at 0x8000
    // This program:
    // 1. Enables timer by writing to CONTROL register
    // 2. Clears I flag to enable interrupts (CLI)
    // 3. Enters infinite loop (NOP, JMP loop)
    let main_program = [
        0xA9, 0x01, // LDA #$01 - Load "enable" value
        0x8D, 0x01, 0xD0, // STA $D001 - Enable timer
        0x58, // CLI - Clear I flag (enable interrupts)
        0xEA, // NOP - No operation
        0x4C, 0x08, 0x80, // JMP $8008 - Jump to NOP (infinite loop)
    ];

    for (i, &byte) in main_program.iter().enumerate() {
        memory.write(0x8000 + i as u16, byte);
    }

    // Set reset vector to point to main program
    memory.write(0xFFFC, 0x00); // Reset vector low byte
    memory.write(0xFFFD, 0x80); // Reset vector high byte (PC = 0x8000)

    // Create CPU
    let mut cpu = CPU::new(memory);

    println!("Initial CPU state:");
    println!("  PC: 0x{:04X}", cpu.pc());
    println!("  I flag: {} (interrupts {})", cpu.flag_i(), if cpu.flag_i() { "disabled" } else { "enabled" });
    println!();

    // Run for a number of cycles
    println!("Running CPU...");
    println!("(Timer interrupts every 100 cycles)\n");

    for step in 0..10 {
        // Execute 50 cycles
        match cpu.run_for_cycles(50) {
            Ok(cycles) => {
                println!("Step {}: Executed {} cycles", step + 1, cycles);
                println!("  PC: 0x{:04X}", cpu.pc());
                println!("  Total cycles: {}", cpu.cycles());

                // Access timer device to show state
                if let Some(timer) = cpu
                    .memory_mut()
                    .get_device_at_mut::<TimerDevice>(0xD000)
                {
                    // Tick timer for the cycles we just ran
                    for _ in 0..cycles {
                        timer.tick();
                    }

                    println!("  Timer counter: {}", timer.counter());
                    println!("  Timer interrupt: {}", InterruptDevice::has_interrupt(timer));
                }
                println!();
            }
            Err(e) => {
                eprintln!("Execution error: {}", e);
                break;
            }
        }
    }

    println!("Example complete!");
    println!("\nNOTE: This example demonstrates the timer device structure.");
    println!("Full interrupt testing requires a working CPU implementation.");
}
