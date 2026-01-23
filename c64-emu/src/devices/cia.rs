//! CIA (MOS 6526) Complex Interface Adapter emulation.
//!
//! The C64 has two CIA chips:
//! - CIA1 ($DC00-$DCFF): Keyboard matrix, joystick ports, IRQ generation
//! - CIA2 ($DD00-$DDFF): IEC bus control, VIC-II bank selection, NMI generation
//!
//! Each CIA provides:
//! - Two 8-bit I/O ports (A and B)
//! - Two 16-bit countdown timers with interrupt capability
//! - Time-of-day clock with alarm
//! - Serial shift register

use lib6502::Device;
use std::any::Any;
use std::cell::Cell;

/// CIA register count (16 registers, but mirrored across 256 bytes).
#[allow(dead_code)]
pub const CIA_REGISTER_COUNT: usize = 16;

/// CIA I/O port state.
#[derive(Debug, Clone, Default)]
pub struct CiaPort {
    /// Output data register.
    pub data: u8,
    /// Data direction register (0=input, 1=output).
    pub ddr: u8,
}

impl CiaPort {
    /// Create a new port with all pins as input.
    pub fn new() -> Self {
        Self { data: 0, ddr: 0 }
    }

    /// Get the effective output value (considering DDR).
    #[inline]
    pub fn output(&self) -> u8 {
        self.data & self.ddr
    }

    /// Read the port, combining output latch and external input.
    #[inline]
    pub fn read(&self, external: u8) -> u8 {
        (self.data & self.ddr) | (external & !self.ddr)
    }
}

/// CIA timer state.
#[derive(Debug, Clone)]
pub struct CiaTimer {
    /// Current countdown value.
    pub counter: u16,
    /// Reload latch value.
    pub latch: u16,
    /// Timer is running.
    pub running: bool,
    /// One-shot mode (stops after underflow).
    pub one_shot: bool,
    /// Timer underflowed this cycle.
    pub underflow: bool,
}

impl CiaTimer {
    /// Create a new timer with default state.
    pub fn new() -> Self {
        Self {
            counter: 0xFFFF,
            latch: 0xFFFF,
            running: false,
            one_shot: false,
            underflow: false,
        }
    }

    /// Clock the timer by one cycle.
    ///
    /// Returns true if the timer underflowed.
    pub fn clock(&mut self) -> bool {
        self.underflow = false;

        if !self.running {
            return false;
        }

        if self.counter == 0 {
            self.underflow = true;
            self.counter = self.latch;

            if self.one_shot {
                self.running = false;
            }
            true
        } else {
            self.counter = self.counter.wrapping_sub(1);
            false
        }
    }

    /// Force reload the counter from latch.
    pub fn force_reload(&mut self) {
        self.counter = self.latch;
    }
}

impl Default for CiaTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Time-of-day clock state.
#[derive(Debug, Clone)]
pub struct TodClock {
    /// Tenths of seconds (BCD 0-9).
    pub tenths: u8,
    /// Seconds (BCD 00-59).
    pub seconds: u8,
    /// Minutes (BCD 00-59).
    pub minutes: u8,
    /// Hours (BCD 01-12, bit 7 = PM).
    pub hours: u8,
    /// Alarm tenths.
    pub alarm_tenths: u8,
    /// Alarm seconds.
    pub alarm_seconds: u8,
    /// Alarm minutes.
    pub alarm_minutes: u8,
    /// Alarm hours.
    pub alarm_hours: u8,
    /// TOD clock is stopped (during write sequence).
    pub stopped: bool,
    /// TOD output is latched (during read sequence).
    pub latched: bool,
    /// Latched values for reading.
    latch_tenths: u8,
    latch_seconds: u8,
    latch_minutes: u8,
}

impl TodClock {
    /// Create a new TOD clock.
    pub fn new() -> Self {
        Self {
            tenths: 0,
            seconds: 0,
            minutes: 0,
            hours: 0x01, // 1:00 AM
            alarm_tenths: 0,
            alarm_seconds: 0,
            alarm_minutes: 0,
            alarm_hours: 0,
            stopped: false,
            latched: false,
            latch_tenths: 0,
            latch_seconds: 0,
            latch_minutes: 0,
        }
    }

    /// Check if current time matches alarm.
    pub fn alarm_match(&self) -> bool {
        self.tenths == self.alarm_tenths
            && self.seconds == self.alarm_seconds
            && self.minutes == self.alarm_minutes
            && self.hours == self.alarm_hours
    }
}

impl Default for TodClock {
    fn default() -> Self {
        Self::new()
    }
}

/// CIA chip type (affects interrupt output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiaType {
    /// CIA1 generates IRQ.
    Cia1,
    /// CIA2 generates NMI.
    Cia2,
}

/// MOS 6526 Complex Interface Adapter.
#[derive(Debug)]
pub struct Cia6526 {
    /// CIA type (CIA1 for IRQ, CIA2 for NMI).
    cia_type: CiaType,

    /// I/O Port A.
    pub port_a: CiaPort,
    /// I/O Port B.
    pub port_b: CiaPort,

    /// Timer A.
    pub timer_a: CiaTimer,
    /// Timer B.
    pub timer_b: CiaTimer,

    /// Time-of-day clock.
    pub tod: TodClock,

    /// Serial data register.
    pub sdr: u8,

    /// Interrupt control register (read: flags, write: mask).
    /// Uses Cell for interior mutability - reading ICR clears flags.
    interrupt_flags: Cell<u8>,
    /// Interrupt mask.
    interrupt_mask: u8,

    /// IRQ/NMI line is active.
    /// Uses Cell for interior mutability - reading ICR clears pending state.
    interrupt_pending: Cell<bool>,

    /// Control register A.
    cra: u8,
    /// Control register B.
    crb: u8,

    /// External input for port A (keyboard rows, joystick).
    pub external_a: u8,
    /// External input for port B (keyboard cols, joystick).
    pub external_b: u8,
}

impl Cia6526 {
    /// Create a new CIA chip.
    pub fn new(cia_type: CiaType) -> Self {
        Self {
            cia_type,
            port_a: CiaPort::new(),
            port_b: CiaPort::new(),
            timer_a: CiaTimer::new(),
            timer_b: CiaTimer::new(),
            tod: TodClock::new(),
            sdr: 0,
            interrupt_flags: Cell::new(0),
            interrupt_mask: 0,
            interrupt_pending: Cell::new(false),
            cra: 0,
            crb: 0,
            external_a: 0xFF,
            external_b: 0xFF,
        }
    }

    /// Create CIA1 (IRQ, keyboard/joystick).
    pub fn new_cia1() -> Self {
        Self::new(CiaType::Cia1)
    }

    /// Create CIA2 (NMI, IEC bus/VIC bank).
    pub fn new_cia2() -> Self {
        Self::new(CiaType::Cia2)
    }

    /// Clock the CIA by one cycle.
    pub fn clock(&mut self) {
        // Clock Timer A
        let timer_a_underflow = self.timer_a.clock();
        if timer_a_underflow {
            self.interrupt_flags.set(self.interrupt_flags.get() | 0x01); // Timer A interrupt flag
            self.check_interrupt();
        }

        // Clock Timer B (can be chained to Timer A)
        let timer_b_input = if self.crb & 0x60 == 0x40 {
            // Timer B counts Timer A underflows
            timer_a_underflow
        } else {
            // Timer B counts clock cycles
            true
        };

        if timer_b_input {
            let timer_b_underflow = self.timer_b.clock();
            if timer_b_underflow {
                self.interrupt_flags.set(self.interrupt_flags.get() | 0x02); // Timer B interrupt flag
                self.check_interrupt();
            }
        }
    }

    /// Check and update interrupt state.
    fn check_interrupt(&mut self) {
        if self.interrupt_flags.get() & self.interrupt_mask != 0 {
            self.interrupt_pending.set(true);
        }
    }

    /// Get the VIC-II bank selection (CIA2 only, from port A bits 0-1).
    ///
    /// Returns 0-3, where the actual VIC bank is (3 - value).
    pub fn vic_bank(&self) -> u8 {
        (!self.port_a.read(self.external_a)) & 0x03
    }

    /// Reset the CIA to power-on state.
    pub fn reset(&mut self) {
        self.port_a = CiaPort::new();
        self.port_b = CiaPort::new();
        self.timer_a = CiaTimer::new();
        self.timer_b = CiaTimer::new();
        self.tod = TodClock::new();
        self.sdr = 0;
        self.interrupt_flags.set(0);
        self.interrupt_mask = 0;
        self.interrupt_pending.set(false);
        self.cra = 0;
        self.crb = 0;
    }

    /// Check if this is CIA1 (IRQ).
    pub fn is_cia1(&self) -> bool {
        self.cia_type == CiaType::Cia1
    }

    /// Check if this is CIA2 (NMI).
    pub fn is_cia2(&self) -> bool {
        self.cia_type == CiaType::Cia2
    }

    /// Set joystick input for port A (active low).
    ///
    /// Bits: 0=up, 1=down, 2=left, 3=right, 4=fire.
    /// Input should use active-low convention (0 = pressed).
    pub fn set_joystick_port_a(&mut self, state: u8) {
        // Joystick overrides bits 0-4 of external input
        self.external_a = (self.external_a & 0xE0) | (!state & 0x1F);
    }

    /// Set joystick input for port B (active low).
    ///
    /// Bits: 0=up, 1=down, 2=left, 3=right, 4=fire.
    /// Input should use active-low convention (0 = pressed).
    pub fn set_joystick_port_b(&mut self, state: u8) {
        // Joystick overrides bits 0-4 of external input
        self.external_b = (self.external_b & 0xE0) | (!state & 0x1F);
    }

    // =========================================================================
    // Save State Accessors
    // =========================================================================

    /// Get TOD latch tenths (for save state).
    pub fn tod_latch_tenths(&self) -> u8 {
        self.tod.latch_tenths
    }

    /// Get TOD latch seconds (for save state).
    pub fn tod_latch_seconds(&self) -> u8 {
        self.tod.latch_seconds
    }

    /// Get TOD latch minutes (for save state).
    pub fn tod_latch_minutes(&self) -> u8 {
        self.tod.latch_minutes
    }

    /// Set TOD latch values (for save state restoration).
    pub fn set_tod_latch(&mut self, tenths: u8, seconds: u8, minutes: u8) {
        self.tod.latch_tenths = tenths;
        self.tod.latch_seconds = seconds;
        self.tod.latch_minutes = minutes;
    }

    /// Get the interrupt flags.
    pub fn interrupt_flags(&self) -> u8 {
        self.interrupt_flags.get()
    }

    /// Set the interrupt flags (for save state restoration).
    pub fn set_interrupt_flags(&mut self, flags: u8) {
        self.interrupt_flags.set(flags);
    }

    /// Get the interrupt mask.
    pub fn interrupt_mask(&self) -> u8 {
        self.interrupt_mask
    }

    /// Set the interrupt mask (for save state restoration).
    pub fn set_interrupt_mask(&mut self, mask: u8) {
        self.interrupt_mask = mask;
    }

    /// Set the interrupt pending flag (for save state restoration).
    pub fn set_interrupt_pending(&mut self, pending: bool) {
        self.interrupt_pending.set(pending);
    }

    /// Get control register A.
    pub fn cra(&self) -> u8 {
        self.cra
    }

    /// Set control register A (for save state restoration).
    pub fn set_cra(&mut self, cra: u8) {
        self.cra = cra;
        // Update timer A state based on CRA
        self.timer_a.running = cra & 0x01 != 0;
        self.timer_a.one_shot = cra & 0x08 != 0;
    }

    /// Get control register B.
    pub fn crb(&self) -> u8 {
        self.crb
    }

    /// Set control register B (for save state restoration).
    pub fn set_crb(&mut self, crb: u8) {
        self.crb = crb;
        // Update timer B state based on CRB
        self.timer_b.running = crb & 0x01 != 0;
        self.timer_b.one_shot = crb & 0x08 != 0;
    }

    /// Get all CIA registers as a 16-byte array.
    ///
    /// Returns the current state of all 16 registers for debugging.
    /// Note: Reading register 0x0D on real hardware clears interrupt flags,
    /// but this method does not have that side effect.
    pub fn get_all_registers(&self) -> [u8; 16] {
        let mut regs = [0u8; 16];

        // Port A/B data
        regs[0x00] = self.port_a.read(self.external_a);
        regs[0x01] = self.port_b.read(self.external_b);
        // Port A/B DDR
        regs[0x02] = self.port_a.ddr;
        regs[0x03] = self.port_b.ddr;
        // Timer A counter
        regs[0x04] = (self.timer_a.counter & 0xFF) as u8;
        regs[0x05] = (self.timer_a.counter >> 8) as u8;
        // Timer B counter
        regs[0x06] = (self.timer_b.counter & 0xFF) as u8;
        regs[0x07] = (self.timer_b.counter >> 8) as u8;
        // TOD (use live values, not latched)
        regs[0x08] = self.tod.tenths;
        regs[0x09] = self.tod.seconds;
        regs[0x0A] = self.tod.minutes;
        regs[0x0B] = self.tod.hours;
        // Serial data register
        regs[0x0C] = self.sdr;
        // Interrupt control (flags with pending bit)
        regs[0x0D] = self.interrupt_flags.get()
            | if self.interrupt_pending.get() {
                0x80
            } else {
                0
            };
        // Control registers
        regs[0x0E] = self.cra;
        regs[0x0F] = self.crb;

        regs
    }
}

impl Device for Cia6526 {
    fn read(&self, offset: u16) -> u8 {
        // Registers mirror every 16 bytes
        match (offset & 0x0F) as usize {
            // Port A data
            0x00 => self.port_a.read(self.external_a),
            // Port B data
            0x01 => self.port_b.read(self.external_b),
            // Port A DDR
            0x02 => self.port_a.ddr,
            // Port B DDR
            0x03 => self.port_b.ddr,
            // Timer A low byte
            0x04 => (self.timer_a.counter & 0xFF) as u8,
            // Timer A high byte
            0x05 => (self.timer_a.counter >> 8) as u8,
            // Timer B low byte
            0x06 => (self.timer_b.counter & 0xFF) as u8,
            // Timer B high byte
            0x07 => (self.timer_b.counter >> 8) as u8,
            // TOD tenths
            0x08 => {
                if self.tod.latched {
                    self.tod.latch_tenths
                } else {
                    self.tod.tenths
                }
            }
            // TOD seconds
            0x09 => {
                if self.tod.latched {
                    self.tod.latch_seconds
                } else {
                    self.tod.seconds
                }
            }
            // TOD minutes
            0x0A => {
                if self.tod.latched {
                    self.tod.latch_minutes
                } else {
                    self.tod.minutes
                }
            }
            // TOD hours (reading latches TOD)
            0x0B => self.tod.hours,
            // Serial data register
            0x0C => self.sdr,
            // Interrupt control (read clears flags)
            // On real hardware, reading ICR clears the interrupt flags
            0x0D => {
                let flags = self.interrupt_flags.get();
                let pending = self.interrupt_pending.get();
                // Clear flags and pending state after reading
                self.interrupt_flags.set(0);
                self.interrupt_pending.set(false);
                flags | if pending { 0x80 } else { 0 }
            }
            // Control register A
            0x0E => self.cra,
            // Control register B
            0x0F => self.crb,
            _ => 0xFF,
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        match (offset & 0x0F) as usize {
            // Port A data
            0x00 => self.port_a.data = value,
            // Port B data
            0x01 => self.port_b.data = value,
            // Port A DDR
            0x02 => self.port_a.ddr = value,
            // Port B DDR
            0x03 => self.port_b.ddr = value,
            // Timer A latch low byte
            0x04 => self.timer_a.latch = (self.timer_a.latch & 0xFF00) | value as u16,
            // Timer A latch high byte (also loads counter if not running)
            0x05 => {
                self.timer_a.latch = (self.timer_a.latch & 0x00FF) | ((value as u16) << 8);
                if !self.timer_a.running {
                    self.timer_a.counter = self.timer_a.latch;
                }
            }
            // Timer B latch low byte
            0x06 => self.timer_b.latch = (self.timer_b.latch & 0xFF00) | value as u16,
            // Timer B latch high byte
            0x07 => {
                self.timer_b.latch = (self.timer_b.latch & 0x00FF) | ((value as u16) << 8);
                if !self.timer_b.running {
                    self.timer_b.counter = self.timer_b.latch;
                }
            }
            // TOD tenths / alarm tenths
            0x08 => {
                if self.crb & 0x80 != 0 {
                    self.tod.alarm_tenths = value & 0x0F;
                } else {
                    self.tod.tenths = value & 0x0F;
                    self.tod.stopped = false; // Writing tenths starts TOD
                }
            }
            // TOD seconds / alarm seconds
            0x09 => {
                if self.crb & 0x80 != 0 {
                    self.tod.alarm_seconds = value & 0x7F;
                } else {
                    self.tod.seconds = value & 0x7F;
                }
            }
            // TOD minutes / alarm minutes
            0x0A => {
                if self.crb & 0x80 != 0 {
                    self.tod.alarm_minutes = value & 0x7F;
                } else {
                    self.tod.minutes = value & 0x7F;
                }
            }
            // TOD hours / alarm hours (writing hours stops TOD)
            0x0B => {
                if self.crb & 0x80 != 0 {
                    self.tod.alarm_hours = value & 0x9F;
                } else {
                    self.tod.hours = value & 0x9F;
                    self.tod.stopped = true;
                }
            }
            // Serial data register
            0x0C => self.sdr = value,
            // Interrupt control (write sets/clears mask)
            0x0D => {
                let mask = value & 0x1F;
                if value & 0x80 != 0 {
                    // Set bits
                    self.interrupt_mask |= mask;
                } else {
                    // Clear bits
                    self.interrupt_mask &= !mask;
                }
                self.check_interrupt();
            }
            // Control register A
            0x0E => {
                self.cra = value;
                self.timer_a.running = value & 0x01 != 0;
                self.timer_a.one_shot = value & 0x08 != 0;
                if value & 0x10 != 0 {
                    // Force load
                    self.timer_a.force_reload();
                }
            }
            // Control register B
            0x0F => {
                self.crb = value;
                self.timer_b.running = value & 0x01 != 0;
                self.timer_b.one_shot = value & 0x08 != 0;
                if value & 0x10 != 0 {
                    // Force load
                    self.timer_b.force_reload();
                }
            }
            _ => {}
        }
    }

    fn size(&self) -> u16 {
        256 // CIA registers mirror across the full page
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn has_interrupt(&self) -> bool {
        self.interrupt_pending.get() && self.cia_type == CiaType::Cia1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cia() {
        let cia1 = Cia6526::new_cia1();
        assert!(cia1.is_cia1());
        assert!(!cia1.interrupt_pending.get());

        let cia2 = Cia6526::new_cia2();
        assert!(cia2.is_cia2());
    }

    #[test]
    fn test_port_read_write() {
        let mut cia = Cia6526::new_cia1();

        // Set port A as all outputs
        cia.write(0x02, 0xFF);
        cia.write(0x00, 0x55);
        assert_eq!(cia.read(0x00), 0x55);

        // Set port A as all inputs
        cia.write(0x02, 0x00);
        cia.external_a = 0xAA;
        assert_eq!(cia.read(0x00), 0xAA);
    }

    #[test]
    fn test_timer_countdown() {
        let mut cia = Cia6526::new_cia1();

        // Set timer A to count down from 5
        cia.write(0x04, 0x05);
        cia.write(0x05, 0x00);

        // Start timer
        cia.write(0x0E, 0x01);

        // Clock 5 times - should count: 5->4->3->2->1->0
        for _ in 0..5 {
            cia.clock();
        }
        // After 5 clocks, counter should be at 0 but not yet underflowed
        assert_eq!(cia.timer_a.counter, 0);

        // Clock one more time - underflow happens when counter is at 0
        cia.clock();
        // Timer should have underflowed and set interrupt flag
        assert!(cia.interrupt_flags.get() & 0x01 != 0);
    }

    #[test]
    fn test_interrupt_mask() {
        let mut cia = Cia6526::new_cia1();

        // Set interrupt mask for Timer A
        cia.write(0x0D, 0x81); // Set bit 0

        // Set timer for quick underflow
        cia.write(0x04, 0x01);
        cia.write(0x05, 0x00);
        cia.write(0x0E, 0x01);

        // Clock until underflow
        cia.clock();
        cia.clock();

        // Should have interrupt
        assert!(cia.interrupt_pending.get());
    }

    #[test]
    fn test_vic_bank_selection() {
        let mut cia = Cia6526::new_cia2();

        // Port A bits 0-1 control VIC bank (active low)
        cia.port_a.ddr = 0x03;
        cia.port_a.data = 0x00; // Both bits low = VIC bank 3
        assert_eq!(cia.vic_bank(), 3);

        cia.port_a.data = 0x03; // Both bits high = VIC bank 0
        assert_eq!(cia.vic_bank(), 0);
    }

    #[test]
    fn test_size() {
        let cia = Cia6526::new_cia1();
        assert_eq!(cia.size(), 256);
    }
}
