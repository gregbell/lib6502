# Quickstart: Implementing Interrupt-Capable Devices

**Date**: 2025-11-18
**Audience**: Developers implementing devices that signal interrupts to the CPU

## Overview

This guide shows how to implement a device that can signal interrupts to the 6502 CPU emulator. You'll create memory-mapped registers that the CPU's interrupt service routine (ISR) can poll and acknowledge.

## Prerequisites

- Familiarity with the 6502 instruction set and interrupt handling
- Understanding of memory-mapped I/O concepts
- Rust programming knowledge (for native devices)

## Quick Example: Timer Device

Here's a complete interrupt-capable timer device:

```rust
use lib6502::{MemoryBus, InterruptDevice};

pub struct TimerDevice {
    /// Base address for memory-mapped registers
    base_address: u16,

    /// Interrupt pending flag (contributes to CPU IRQ line)
    interrupt_pending: bool,

    /// Timer counter value
    counter: u16,

    /// Timer reload value
    reload_value: u16,

    /// Interrupt enable flag
    interrupt_enabled: bool,
}

impl TimerDevice {
    pub fn new(base_address: u16) -> Self {
        Self {
            base_address,
            interrupt_pending: false,
            counter: 0,
            reload_value: 0,
            interrupt_enabled: false,
        }
    }

    /// Tick the timer forward one cycle
    pub fn tick(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
            if self.counter == 0 {
                // Timer expired - trigger interrupt
                if self.interrupt_enabled {
                    self.interrupt_pending = true;
                }
                // Reload counter for next interval
                self.counter = self.reload_value;
            }
        }
    }
}

impl InterruptDevice for TimerDevice {
    fn has_interrupt(&self) -> bool {
        self.interrupt_pending
    }

    fn address_range(&self) -> (u16, u16) {
        // Timer uses 4 bytes: status, control, counter_lo, counter_hi
        (self.base_address, self.base_address + 3)
    }
}

impl MemoryBus for TimerDevice {
    fn read(&self, addr: u16) -> u8 {
        match addr - self.base_address {
            0 => {
                // STATUS register (read-only)
                let mut status = 0;
                if self.interrupt_pending {
                    status |= 0x80; // Bit 7: Interrupt pending
                }
                if self.interrupt_enabled {
                    status |= 0x40; // Bit 6: Interrupt enabled
                }
                status
            }
            1 => {
                // CONTROL register (read-write)
                let mut control = 0;
                if self.interrupt_enabled {
                    control |= 0x80; // Bit 7: Interrupt enable
                }
                control
            }
            2 => {
                // COUNTER_LO (read-only)
                (self.counter & 0xFF) as u8
            }
            3 => {
                // COUNTER_HI (read-only)
                (self.counter >> 8) as u8
            }
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr - self.base_address {
            0 => {
                // STATUS register (write clears interrupt)
                self.interrupt_pending = false;
            }
            1 => {
                // CONTROL register
                self.interrupt_enabled = (value & 0x80) != 0;
            }
            2 => {
                // RELOAD_LO (write-only)
                self.reload_value = (self.reload_value & 0xFF00) | (value as u16);
                self.counter = self.reload_value;
            }
            3 => {
                // RELOAD_HI (write-only)
                self.reload_value = (self.reload_value & 0x00FF) | ((value as u16) << 8);
                self.counter = self.reload_value;
            }
            _ => {}
        }
    }
}
```

## Step-by-Step Guide

### Step 1: Define Your Device Structure

```rust
pub struct MyDevice {
    /// Base address for memory-mapped registers
    base_address: u16,

    /// REQUIRED: Interrupt pending flag
    interrupt_pending: bool,

    /// Your device-specific state
    // ... other fields ...
}
```

**Key Points**:
- `base_address`: Where your device's registers appear in memory
- `interrupt_pending`: MUST reflect if device has unserviced interrupt
- Device-specific state: Whatever your device needs to function

### Step 2: Implement InterruptDevice Trait

```rust
impl InterruptDevice for MyDevice {
    fn has_interrupt(&self) -> bool {
        self.interrupt_pending
    }

    fn address_range(&self) -> (u16, u16) {
        // Return (start, end_inclusive) for your register range
        (self.base_address, self.base_address + N)
    }
}
```

**Requirements**:
- `has_interrupt()` MUST return current `interrupt_pending` value
- `address_range()` MUST return unique range not overlapping other devices
- Range size depends on how many registers your device needs

### Step 3: Implement MemoryBus Trait

```rust
impl MemoryBus for MyDevice {
    fn read(&self, addr: u16) -> u8 {
        match addr - self.base_address {
            0 => {
                // STATUS register
                let mut status = 0;
                if self.interrupt_pending {
                    status |= 0x80; // Bit 7: Interrupt flag
                }
                // Add other status bits as needed
                status
            }
            // ... other registers ...
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr - self.base_address {
            0 => {
                // Writing to STATUS clears interrupt
                self.interrupt_pending = false;
            }
            1 => {
                // CONTROL register
                // ... device-specific logic ...
            }
            // ... other registers ...
            _ => {}
        }
    }
}
```

**Requirements**:
- STATUS register SHOULD expose interrupt_pending in bit 7
- Writing to a register (STATUS or CONTROL) MUST clear interrupt_pending
- Registers MUST be accessible at base_address + offset

### Step 4: Trigger Interrupts from Device Logic

```rust
impl MyDevice {
    pub fn update(&mut self) {
        // Your device update logic
        // When interrupt condition occurs:
        if some_interrupt_condition {
            self.interrupt_pending = true;
        }
    }
}
```

**Key Points**:
- Set `interrupt_pending = true` when device needs CPU attention
- CPU will check IRQ line after each instruction
- ISR will eventually poll your device's STATUS register

### Step 5: Integrate with Memory Mapper

```rust
pub struct MemoryMapper {
    devices: Vec<Box<dyn InterruptDevice + MemoryBus>>,
}

impl MemoryMapper {
    pub fn add_device(&mut self, device: Box<dyn InterruptDevice + MemoryBus>) {
        // Validate address range doesn't overlap
        let new_range = device.address_range();
        for existing in &self.devices {
            let existing_range = existing.address_range();
            if ranges_overlap(new_range, existing_range) {
                panic!("Device address ranges overlap!");
            }
        }
        self.devices.push(device);
    }

    pub fn irq_active(&self) -> bool {
        self.devices.iter().any(|d| d.has_interrupt())
    }
}

impl MemoryBus for MemoryMapper {
    fn read(&self, addr: u16) -> u8 {
        for device in &self.devices {
            let (start, end) = device.address_range();
            if addr >= start && addr <= end {
                return device.read(addr);
            }
        }
        // Fall through to RAM or other memory
        0
    }

    fn write(&mut self, addr: u16, value: u8) {
        for device in &mut self.devices {
            let (start, end) = device.address_range();
            if addr >= start && addr <= end {
                device.write(addr, value);
                return;
            }
        }
        // Fall through to RAM or other memory
    }
}
```

---

## Writing the ISR (6502 Assembly)

Your interrupt service routine needs to poll your device's STATUS register:

```asm
    .org $8000
irq_handler:
    ; Save registers
    pha                ; Push A

    ; Poll your device at base address (e.g., $D000)
    lda $D000          ; Read STATUS register
    and #$80           ; Test bit 7 (interrupt pending)
    beq not_my_device  ; If clear, not this device

    ; Handle your device's interrupt
    ; ... device-specific handling ...

    ; Acknowledge interrupt by writing to STATUS or CONTROL
    lda #$00
    sta $D000          ; Write to STATUS clears interrupt

not_my_device:
    ; Check other devices if needed
    ; ...

    ; Restore registers and return
    pla                ; Pop A
    rti                ; Return from interrupt

    .org $FFFE
    .word irq_handler  ; Set IRQ vector
```

**Key Points**:
- Always save/restore registers (A, X, Y) in ISR
- Read STATUS register to check if your device triggered interrupt
- Perform device-specific handling
- Clear interrupt by writing to appropriate register
- Use RTI (not RTS) to return from interrupt

---

## Testing Your Device

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_triggered() {
        let mut device = MyDevice::new(0xD000);

        // Initially no interrupt
        assert!(!device.has_interrupt());

        // Trigger interrupt condition
        device.trigger_interrupt();

        // Interrupt should be pending
        assert!(device.has_interrupt());
    }

    #[test]
    fn test_interrupt_cleared_on_status_write() {
        let mut device = MyDevice::new(0xD000);

        // Set interrupt
        device.trigger_interrupt();
        assert!(device.has_interrupt());

        // Write to STATUS register (base + 0)
        device.write(0xD000, 0x00);

        // Interrupt should be cleared
        assert!(!device.has_interrupt());
    }

    #[test]
    fn test_status_register_reflects_interrupt() {
        let mut device = MyDevice::new(0xD000);

        // No interrupt - bit 7 should be clear
        let status = device.read(0xD000);
        assert_eq!(status & 0x80, 0);

        // Set interrupt
        device.trigger_interrupt();

        // Bit 7 should be set
        let status = device.read(0xD000);
        assert_eq!(status & 0x80, 0x80);
    }
}
```

### Integration Tests with CPU

```rust
#[test]
fn test_cpu_services_interrupt() {
    // Set up CPU with your device
    let mut memory = MemoryMapper::new();
    memory.add_device(Box::new(MyDevice::new(0xD000)));

    // Set up IRQ vector to point to handler
    memory.write(0xFFFE, 0x00); // Handler at $8000
    memory.write(0xFFFF, 0x80);

    // Create CPU
    let mut cpu = CPU::new(memory);
    cpu.set_flag_i(false); // Enable interrupts

    // Load simple handler that acknowledges interrupt
    // LDA $D000 ; Read status
    // STA $D000 ; Clear interrupt
    // RTI
    memory.write(0x8000, 0xAD); // LDA absolute
    memory.write(0x8001, 0x00);
    memory.write(0x8002, 0xD0);
    memory.write(0x8003, 0x8D); // STA absolute
    memory.write(0x8004, 0x00);
    memory.write(0x8005, 0xD0);
    memory.write(0x8006, 0x40); // RTI

    // Trigger device interrupt
    cpu.memory_mut().device_at_mut(0xD000).trigger_interrupt();

    // CPU should be at some address initially
    let original_pc = cpu.pc();

    // Execute one instruction
    cpu.step().unwrap();

    // CPU should have serviced interrupt and jumped to handler
    assert_eq!(cpu.pc(), 0x8000);
    assert!(cpu.flag_i()); // I flag should be set

    // Execute handler instructions
    cpu.step().unwrap(); // LDA $D000
    cpu.step().unwrap(); // STA $D000
    cpu.step().unwrap(); // RTI

    // Should have returned near original PC
    assert_eq!(cpu.pc(), original_pc);
    assert!(!cpu.flag_i()); // I flag restored

    // Device interrupt should be cleared
    assert!(!cpu.memory().device_at(0xD000).has_interrupt());
}
```

---

## Common Patterns

### Pattern 1: Read-to-Acknowledge

Device clears interrupt when ISR reads STATUS register:

```rust
fn read(&self, addr: u16) -> u8 {
    match addr - self.base_address {
        0 => {
            let status = if self.interrupt_pending { 0x80 } else { 0 };
            // Clear interrupt on read
            self.interrupt_pending = false;
            status
        }
        _ => 0,
    }
}
```

### Pattern 2: Write-to-Acknowledge

Device clears interrupt when ISR writes to STATUS or CONTROL:

```rust
fn write(&mut self, addr: u16, value: u8) {
    match addr - self.base_address {
        0 => {
            // Any write to STATUS clears interrupt
            self.interrupt_pending = false;
        }
        _ => {}
    }
}
```

### Pattern 3: Data-Read-Clears

Device clears interrupt when ISR reads data register:

```rust
fn read(&self, addr: u16) -> u8 {
    match addr - self.base_address {
        0 => {
            // STATUS register (read-only, doesn't clear)
            if self.interrupt_pending { 0x80 } else { 0 }
        }
        1 => {
            // DATA register (read clears interrupt)
            let data = self.data_buffer;
            self.interrupt_pending = false;
            data
        }
        _ => 0,
    }
}
```

---

## Best Practices

### DO:
- ✅ Set `interrupt_pending = true` when device needs CPU attention
- ✅ Clear `interrupt_pending = false` when ISR acknowledges
- ✅ Expose interrupt status in bit 7 of STATUS register (by convention)
- ✅ Document your register layout clearly
- ✅ Validate address range doesn't overlap other devices
- ✅ Test interrupt behavior in isolation and with CPU

### DON'T:
- ❌ Don't panic in `read()` or `write()` methods
- ❌ Don't hold `interrupt_pending = true` forever (ISR must be able to clear)
- ❌ Don't clear interrupt before ISR has a chance to handle it
- ❌ Don't assume ISR will poll in specific order (make it deterministic)
- ❌ Don't use overlapping address ranges with other devices

---

## Troubleshooting

### Interrupt Never Fires

**Symptom**: Device sets `interrupt_pending = true` but ISR never executes

**Possible Causes**:
1. I flag is set (interrupts disabled) - Check `cpu.flag_i()`
2. IRQ vector not set - Check bytes at 0xFFFE-0xFFFF
3. Device address range not registered with memory mapper
4. `irq_active()` method not checking your device

**Solution**: Add logging to verify:
- Device sets `interrupt_pending = true`
- CPU's `irq_active()` returns true
- CPU's I flag is false
- IRQ vector points to valid handler

### Interrupt Fires but Never Clears

**Symptom**: ISR executes repeatedly in infinite loop

**Possible Causes**:
1. ISR not writing to device register to clear interrupt
2. Device not clearing `interrupt_pending` on register write
3. Device keeps re-triggering interrupt faster than ISR clears it

**Solution**:
- Add logging to ISR to verify register writes
- Add logging to device `write()` to verify `interrupt_pending` cleared
- Check device logic to ensure interrupt condition is cleared

### Multiple Devices Interfere

**Symptom**: ISR works for one device but fails with multiple devices

**Possible Causes**:
1. Address ranges overlap
2. ISR polls devices in wrong order
3. One device's clear operation affects another device

**Solution**:
- Validate address ranges at startup (no overlap)
- Make ISR poll all devices regardless of order
- Ensure each device's register space is independent

---

## Advanced Topics

### Interrupt Enable/Disable Control

Allow software to enable/disable interrupts per-device:

```rust
impl MemoryBus for MyDevice {
    fn write(&mut self, addr: u16, value: u8) {
        match addr - self.base_address {
            1 => {
                // CONTROL register
                self.interrupt_enabled = (value & 0x80) != 0;

                // Clear interrupt if disabled
                if !self.interrupt_enabled {
                    self.interrupt_pending = false;
                }
            }
            _ => {}
        }
    }
}
```

### Edge vs Level Triggering

Level-triggered (default): Interrupt remains active until cleared
Edge-triggered: Interrupt fires once per event, auto-clears

```rust
// Edge-triggered example
pub fn trigger_interrupt(&mut self) {
    if !self.interrupt_pending {
        // Only trigger if not already pending
        self.interrupt_pending = true;
    }
}
```

### Interrupt Priority (Multiple Devices)

ISR can implement priority by polling order:

```asm
irq_handler:
    ; Always check high-priority device first
    lda $D000          ; Timer (high priority)
    and #$80
    bne handle_timer

    lda $D100          ; UART (medium priority)
    and #$80
    bne handle_uart

    lda $D200          ; Keyboard (low priority)
    and #$80
    bne handle_keyboard

    rti                ; No devices had interrupts
```

---

## Next Steps

1. **Implement your device** following the timer example
2. **Write unit tests** for interrupt behavior
3. **Create integration tests** with CPU
4. **Write ISR in assembly** to poll and acknowledge
5. **Test in real scenarios** with multiple devices

For more details, see:
- [data-model.md](./data-model.md) - Interrupt state machine and validation rules
- [contracts/cpu-irq-api.md](./contracts/cpu-irq-api.md) - Formal interface contracts
- [../plan.md](./plan.md) - Implementation plan and constitution check
