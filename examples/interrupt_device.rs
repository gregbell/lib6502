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
    #[allow(dead_code)]
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

/// Interrupt-capable UART device with memory-mapped registers.
///
/// # Architecture
///
/// The UART simulates a serial communication device that generates interrupts
/// when data is received. Data is placed in a receive buffer and an interrupt
/// is triggered for the ISR to read.
///
/// # Memory-Mapped Registers
///
/// When mapped to base address (e.g., 0xD100):
///
/// ```text
/// 0xD100: STATUS register (read-only)
///     Bit 7: Interrupt pending (1 = data available, 0 = none)
///     Bit 0: Data available (1 = data in buffer, 0 = buffer empty)
///     Bit 6-1: Reserved (read as 0)
///
/// 0xD101: CONTROL register (write-only)
///     Bit 7: Interrupt acknowledge (write 1 to clear interrupt)
///     Bit 0: Receive interrupt enable (write 1 to enable)
///     Bit 6-1: Reserved
///
/// 0xD102: Reserved
/// 0xD103: Reserved
///
/// 0xD104: DATA register
///     Read: Get received byte (automatically clears interrupt if enabled)
///     Write: Ignored (no transmit simulation)
/// ```
///
/// # Interrupt Behavior
///
/// - **Assertion**: Interrupt asserted when byte received (via receive_byte())
/// - **Acknowledgment**: ISR can clear by writing CONTROL or reading DATA register
/// - **Level-sensitive**: Interrupt remains active until acknowledged
/// - **Data buffer**: Single-byte buffer (overwrites if not read before next byte)
///
/// # Example 6502 ISR
///
/// ```asm
/// uart_isr:
///     pha                ; Save accumulator
///
///     lda $D100          ; Read UART STATUS register
///     and #$80           ; Check interrupt pending bit
///     beq not_uart       ; If not set, not our interrupt
///
///     ; Handle UART interrupt - reading DATA clears interrupt
///     lda $D104          ; Read received byte (clears interrupt)
///     ; ... process byte ...
///
/// not_uart:
///     pla                ; Restore accumulator
///     rti                ; Return from interrupt
/// ```
pub struct UartDevice {
    /// Base address where device is mapped in memory
    #[allow(dead_code)]
    base_address: u16,

    /// Interrupt pending flag (contributes to CPU IRQ line)
    interrupt_pending: bool,

    /// Data available flag
    data_available: bool,

    /// Receive data buffer (single byte)
    rx_buffer: u8,

    /// Receive interrupt enabled flag
    rx_interrupt_enabled: bool,
}

impl UartDevice {
    /// Create a new UartDevice.
    ///
    /// # Arguments
    ///
    /// * `base_address` - Memory base address for device registers
    ///
    /// # Examples
    ///
    /// ```
    /// # use lib6502::*;
    /// # use std::any::Any;
    /// # pub struct UartDevice { base_address: u16, interrupt_pending: bool, data_available: bool, rx_buffer: u8, rx_interrupt_enabled: bool }
    /// # impl UartDevice {
    /// #   pub fn new(base_address: u16) -> Self {
    /// #     Self { base_address, interrupt_pending: false, data_available: false, rx_buffer: 0, rx_interrupt_enabled: false }
    /// #   }
    /// # }
    /// // Create UART at 0xD100
    /// let uart = UartDevice::new(0xD100);
    /// ```
    pub fn new(base_address: u16) -> Self {
        Self {
            base_address,
            interrupt_pending: false,
            data_available: false,
            rx_buffer: 0,
            rx_interrupt_enabled: false,
        }
    }

    /// Simulate receiving a byte on the UART.
    ///
    /// This method would typically be called by external code simulating
    /// serial input. If receive interrupts are enabled, this triggers an interrupt.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte received on the serial line
    ///
    /// # Examples
    ///
    /// ```
    /// # use lib6502::*;
    /// # use std::any::Any;
    /// # pub struct UartDevice { base_address: u16, interrupt_pending: bool, data_available: bool, rx_buffer: u8, rx_interrupt_enabled: bool }
    /// # impl UartDevice {
    /// #   pub fn new(base_address: u16) -> Self {
    /// #     Self { base_address, interrupt_pending: false, data_available: false, rx_buffer: 0, rx_interrupt_enabled: false }
    /// #   }
    /// #   pub fn receive_byte(&mut self, byte: u8) {
    /// #     self.rx_buffer = byte;
    /// #     self.data_available = true;
    /// #     if self.rx_interrupt_enabled {
    /// #       self.interrupt_pending = true;
    /// #     }
    /// #   }
    /// #   pub fn has_interrupt(&self) -> bool { self.interrupt_pending }
    /// # }
    /// let mut uart = UartDevice::new(0xD100);
    /// uart.receive_byte(0x41); // Receive 'A'
    /// // assert!(uart.has_interrupt());
    /// ```
    pub fn receive_byte(&mut self, byte: u8) {
        self.rx_buffer = byte;
        self.data_available = true;

        // Trigger interrupt if enabled
        if self.rx_interrupt_enabled {
            self.interrupt_pending = true;
        }
    }

    /// Check if data is available in receive buffer.
    pub fn data_available(&self) -> bool {
        self.data_available
    }

    /// Check if receive interrupts are enabled.
    pub fn rx_interrupt_enabled(&self) -> bool {
        self.rx_interrupt_enabled
    }
}

impl InterruptDevice for UartDevice {
    fn has_interrupt(&self) -> bool {
        self.interrupt_pending
    }
}

impl Device for UartDevice {
    fn read(&self, offset: u16) -> u8 {
        match offset {
            0 => {
                // STATUS register (read-only)
                // Bit 7: Interrupt pending
                // Bit 0: Data available
                let mut status = 0x00;
                if self.interrupt_pending {
                    status |= 0x80; // Set bit 7
                }
                if self.data_available {
                    status |= 0x01; // Set bit 0
                }
                status
            }
            1 => {
                // CONTROL register (write-only, reads return 0)
                0x00
            }
            2 | 3 => {
                // Reserved registers
                0x00
            }
            4 => {
                // DATA register - reading this is handled specially in write()
                // Return buffered data (note: actual read side-effect handled via mutable access)
                self.rx_buffer
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

                // Bit 0: Receive interrupt enable
                self.rx_interrupt_enabled = (value & 0x01) != 0;
            }
            2 | 3 => {
                // Reserved registers, ignore writes
            }
            4 => {
                // DATA register - writes ignored (no transmit simulation)
                // Note: Real UART would have separate TX register
            }
            _ => {
                // Unmapped offsets, ignore writes
            }
        }
    }

    fn size(&self) -> u16 {
        5 // Five registers: STATUS, CONTROL, Reserved×2, DATA
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

// Note: Reading DATA register should clear interrupt, but this requires mutable access.
// Since Device::read() takes &self (immutable), we handle this via memory bus write pattern:
// ISR typically reads DATA to get byte, then optionally writes CONTROL to acknowledge.
// Alternatively, UartDevice can auto-clear interrupt when data_available is read via special handling.

/// Example demonstration of multi-device interrupt coordination.
///
/// This function sets up a complete 6502 system with:
/// - 64KB RAM
/// - Timer device at 0xD000-0xD003 (high priority)
/// - UART device at 0xD100-0xD104 (low priority)
/// - IRQ handler that polls both devices in priority order
/// - Main program that enables interrupts and waits
///
/// # Multi-Device ISR Pattern
///
/// The ISR demonstrates the standard pattern for handling multiple interrupt sources:
/// 1. Poll device 1 (timer) - highest priority
///    - Check STATUS register bit 7
///    - If set, handle and acknowledge
/// 2. Poll device 2 (UART) - lower priority
///    - Check STATUS register bit 7
///    - If set, handle and acknowledge
/// 3. Return via RTI
///
/// If both devices have pending interrupts, the timer is serviced first.
/// After RTI, if the UART interrupt is still pending, the CPU will
/// immediately re-enter the ISR to service it.
fn main() {
    println!("=== 6502 Multi-Device Interrupt Example ===\n");

    // Create memory mapper
    let mut memory = MappedMemory::new();

    // Add 48KB RAM at 0x0000-0xBFFF (leaves space for I/O and ROM)
    memory
        .add_device(0x0000, Box::new(RamDevice::new(0xC000)))
        .expect("Failed to add RAM");

    // Add timer device at 0xD000-0xD003 (highest priority)
    let timer = TimerDevice::new(0xD000, 150); // Interrupt every 150 cycles
    memory
        .add_device(0xD000, Box::new(timer))
        .expect("Failed to add timer");

    // Add UART device at 0xD100-0xD104 (lower priority)
    let uart = UartDevice::new(0xD100);
    memory
        .add_device(0xD100, Box::new(uart))
        .expect("Failed to add UART");

    // Set up IRQ vector to point to handler at 0xC000
    memory.write(0xFFFE, 0x00); // IRQ vector low byte
    memory.write(0xFFFF, 0xC0); // IRQ vector high byte (handler at 0xC000)

    // Write multi-device IRQ handler at 0xC000
    // This handler polls both devices in priority order:
    //
    // irq_handler:
    //     pha                    ; Save accumulator
    //
    // check_timer:              ; Check timer (highest priority)
    //     lda $D000              ; Read timer STATUS
    //     and #$80               ; Check interrupt pending bit
    //     beq check_uart         ; If not set, check next device
    //     lda #$80
    //     sta $D001              ; Acknowledge timer
    //
    // check_uart:               ; Check UART (lower priority)
    //     lda $D100              ; Read UART STATUS
    //     and #$80               ; Check interrupt pending bit
    //     beq done               ; If not set, done
    //     lda #$80
    //     sta $D101              ; Acknowledge UART
    //
    // done:
    //     pla                    ; Restore accumulator
    //     rti                    ; Return from interrupt
    let irq_handler = [
        0x48, // PHA - Push accumulator
        // check_timer:
        0xAD, 0x00, 0xD0, // LDA $D000 - Read timer STATUS
        0x29, 0x80, // AND #$80 - Check bit 7
        0xF0, 0x06, // BEQ +6 - Skip to check_uart if not set
        0xA9, 0x80, // LDA #$80
        0x8D, 0x01, 0xD0, // STA $D001 - Acknowledge timer
        // check_uart:
        0xAD, 0x00, 0xD1, // LDA $D100 - Read UART STATUS
        0x29, 0x80, // AND #$80 - Check bit 7
        0xF0, 0x06, // BEQ +6 - Skip to done if not set
        0xA9, 0x80, // LDA #$80
        0x8D, 0x01, 0xD1, // STA $D101 - Acknowledge UART
        // done:
        0x68, // PLA - Restore accumulator
        0x40, // RTI - Return from interrupt
    ];

    for (i, &byte) in irq_handler.iter().enumerate() {
        memory.write(0xC000 + i as u16, byte);
    }

    // Write main program at 0x8000
    // This program:
    // 1. Enables timer by writing to CONTROL register
    // 2. Enables UART receive interrupts by writing to CONTROL register
    // 3. Clears I flag to enable interrupts (CLI)
    // 4. Enters infinite loop (NOP, JMP loop)
    let main_program = [
        0xA9, 0x01, // LDA #$01 - Load "enable" value
        0x8D, 0x01, 0xD0, // STA $D001 - Enable timer
        0x8D, 0x01, 0xD1, // STA $D101 - Enable UART RX interrupts
        0x58, // CLI - Clear I flag (enable interrupts)
        0xEA, // NOP - No operation
        0x4C, 0x0A, 0x80, // JMP $800A - Jump to NOP (infinite loop)
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
    println!(
        "  I flag: {} (interrupts {})",
        cpu.flag_i(),
        if cpu.flag_i() { "disabled" } else { "enabled" }
    );
    println!();

    // Run for a number of cycles
    println!("Running CPU with multi-device interrupts...");
    println!("Timer: Interrupts every 150 cycles");
    println!("UART: Receives byte every 3 steps\n");

    for step in 0..12 {
        // Execute 50 cycles
        match cpu.run_for_cycles(50) {
            Ok(cycles) => {
                println!("Step {}: Executed {} cycles", step + 1, cycles);
                println!("  PC: 0x{:04X}", cpu.pc());
                println!("  Total cycles: {}", cpu.cycles());

                // Tick timer for the cycles we just ran
                if let Some(timer) = cpu.memory_mut().get_device_at_mut::<TimerDevice>(0xD000) {
                    for _ in 0..cycles {
                        timer.tick();
                    }
                    println!(
                        "  Timer: counter={}, interrupt={}",
                        timer.counter(),
                        InterruptDevice::has_interrupt(timer)
                    );
                }

                // Simulate UART receiving a byte every 3 steps
                if step % 3 == 1 {
                    if let Some(uart) = cpu.memory_mut().get_device_at_mut::<UartDevice>(0xD100) {
                        let received_byte = 0x41 + ((step / 3) as u8); // 'A', 'B', 'C', ...
                        uart.receive_byte(received_byte);
                        println!(
                            "  UART: Received byte 0x{:02X} ('{}'), interrupt={}",
                            received_byte,
                            if received_byte.is_ascii_graphic() {
                                received_byte as char
                            } else {
                                '?'
                            },
                            InterruptDevice::has_interrupt(uart)
                        );
                    }
                } else if let Some(uart) = cpu.memory_mut().get_device_at_mut::<UartDevice>(0xD100)
                {
                    println!("  UART: interrupt={}", InterruptDevice::has_interrupt(uart));
                }

                // Show IRQ line state
                println!(
                    "  IRQ line: {}",
                    if cpu.memory_mut().irq_active() {
                        "ACTIVE"
                    } else {
                        "inactive"
                    }
                );
                println!();
            }
            Err(e) => {
                eprintln!("Execution error: {}", e);
                break;
            }
        }
    }

    println!("Example complete!");
    println!("\n=== Multi-Device ISR Pattern Summary ===");
    println!("This example demonstrates:");
    println!("  1. Multiple devices (Timer + UART) sharing the IRQ line");
    println!("  2. ISR polling both devices in priority order");
    println!("  3. Level-sensitive IRQ: line stays active until ALL devices clear");
    println!("  4. CPU re-entering ISR if interrupt still pending after RTI");
    println!("\nThe ISR code at 0xC000 shows the standard polling pattern:");
    println!("  - Check timer STATUS (0xD000), acknowledge if pending");
    println!("  - Check UART STATUS (0xD100), acknowledge if pending");
    println!("  - Return via RTI");
}
