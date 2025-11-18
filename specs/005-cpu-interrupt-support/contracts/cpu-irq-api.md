# Contract: CPU IRQ Line Interface

**Version**: 1.0.0
**Date**: 2025-11-18
**Status**: Draft

## Overview

This contract defines the interface between the CPU and interrupt-capable devices. The CPU checks the IRQ line state at instruction boundaries and services interrupts when appropriate. Devices contribute to the IRQ line through memory-mapped register implementations.

## CPU Interface

### IRQ Line Query

**Purpose**: Check if any device has a pending interrupt request.

**Method Signature**:
```rust
pub trait MemoryBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    /// Check if IRQ line is active (any device has pending interrupt)
    /// Returns true if at least one device has an unserviced interrupt request
    fn irq_active(&self) -> bool;
}
```

**Preconditions**:
- None (always safe to call)

**Postconditions**:
- Returns `true` if and only if at least one device has `interrupt_pending == true`
- Returns `false` if all devices have `interrupt_pending == false`
- Does NOT modify any device state
- Deterministic - calling twice without state change returns same result

**Performance**:
- MUST be O(1) or O(n) where n = number of devices
- SHOULD NOT allocate memory
- SHOULD NOT perform I/O operations

**Example Implementation**:
```rust
impl MemoryBus for MemoryMapper {
    fn irq_active(&self) -> bool {
        self.devices.iter().any(|device| device.has_interrupt())
    }
}
```

---

### Interrupt Service Sequence

**Purpose**: Service a pending interrupt request with cycle-accurate 6502 behavior.

**Method Signature** (internal CPU method):
```rust
impl<M: MemoryBus> CPU<M> {
    /// Service a pending interrupt (called when irq_active && !flag_i)
    /// Executes the 7-cycle IRQ sequence and jumps to handler
    fn service_interrupt(&mut self) -> Result<(), CpuError>;
}
```

**Preconditions**:
- `self.memory.irq_active() == true`
- `self.flag_i == false` (I flag clear - interrupts enabled)
- Called at instruction boundary (after instruction completes)

**Postconditions**:
- Stack contains: [original PC high byte] [original PC low byte] [original status register]
- `self.flag_i == true` (I flag set to prevent nested interrupts)
- `self.pc` set to handler address read from vector at 0xFFFE-0xFFFF
- `self.cycles` incremented by exactly 7
- Returns `Ok(())` on success, `Err(CpuError)` if stack overflow or invalid vector

**Cycle Breakdown**:
1. Push PC high byte to stack - 1 cycle
2. Push PC low byte to stack - 1 cycle
3. Push status register to stack - 1 cycle
4. Set I flag (interrupt disable) - 0 cycles (part of push operation)
5. Read IRQ vector low byte from 0xFFFE - 1 cycle
6. Read IRQ vector high byte from 0xFFFF - 1 cycle
7. Set PC to vector address - 2 cycles (internal operation)

**Total**: 7 cycles (matches real 6502 hardware)

**Error Conditions**:
- Stack overflow during push operations → `Err(CpuError::StackOverflow)`
- Invalid vector address (e.g., 0x0000) → Allowed (CPU jumps to 0x0000)

**Example Call Site**:
```rust
impl<M: MemoryBus> CPU<M> {
    pub fn step(&mut self) -> Result<(), CpuError> {
        // Execute current instruction
        self.execute_instruction()?;

        // Check for interrupts at instruction boundary
        if self.memory.irq_active() && !self.flag_i {
            self.service_interrupt()?;
        }

        Ok(())
    }
}
```

---

## Device Interface

### Interrupt-Capable Device Trait

**Purpose**: Define the contract for devices that can signal interrupts.

**Trait Definition**:
```rust
pub trait InterruptDevice {
    /// Check if device has pending interrupt
    fn has_interrupt(&self) -> bool;

    /// Get the memory address range this device responds to
    fn address_range(&self) -> (u16, u16); // (start, end inclusive)
}
```

**Requirements**:
- Devices MUST implement `InterruptDevice` trait
- Devices MUST also implement `MemoryBus` trait (or be wrapped by memory mapper)
- Devices MUST declare unique, non-overlapping address ranges

---

### Device Register Contract

**Purpose**: Define the expected behavior of device memory-mapped registers.

**Status Register** (Read-Only):
```text
Address: Device base + 0 (example)
Read:
  Bit 7: Interrupt pending flag (1 = pending, 0 = none)
  Bit 6-0: Device-specific status bits

Behavior:
  - Reading this register MAY clear interrupt flag (device-specific)
  - Reading this register MUST NOT modify other device state
  - Multiple reads return consistent results until state changes
```

**Control Register** (Write-Only or Read-Write):
```text
Address: Device base + 1 (example)
Write:
  Bit 7: Interrupt enable (1 = enable, 0 = disable)
  Bit 6-0: Device-specific control bits

Behavior:
  - Writing to this register MUST clear interrupt flag if appropriate
  - Writing with bit pattern matching "acknowledge" clears interrupt
  - Device-specific logic determines exact clear condition
```

**Example Implementation**:
```rust
impl MemoryBus for TimerDevice {
    fn read(&self, addr: u16) -> u8 {
        match addr - self.base_address {
            0 => {
                // Read status register
                let mut status = 0;
                if self.interrupt_pending {
                    status |= 0x80; // Set bit 7
                }
                // Optional: Clear interrupt on status read
                // self.interrupt_pending = false;
                status
            }
            _ => 0
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr - self.base_address {
            1 => {
                // Write control register
                // Clear interrupt flag when acknowledged
                if value & 0x80 != 0 {
                    self.interrupt_pending = false;
                }
            }
            _ => {}
        }
    }
}
```

---

## ISR Contract

**Purpose**: Define the expected behavior of interrupt service routines.

**ISR Entry**:
- PC points to address read from IRQ vector (0xFFFE-0xFFFF)
- I flag is set (interrupts disabled)
- Stack contains return address and status register
- ISR MUST NOT assume which device triggered interrupt

**ISR Execution**:
- ISR SHOULD poll all potential interrupt sources by reading status registers
- ISR SHOULD acknowledge interrupts by writing to control registers
- ISR MAY re-enable interrupts (CLI instruction) if nested interrupts desired
- ISR SHOULD complete quickly to avoid starving normal execution

**ISR Exit**:
- ISR MUST exit via RTI instruction
- RTI restores PC and status register (including I flag)
- If IRQ line still active after RTI, CPU immediately re-enters ISR

**Example ISR** (6502 assembly):
```asm
irq_handler:
    ; Save accumulator (X and Y if needed)
    pha

    ; Poll timer device
    lda $D000          ; Read timer status register
    and #$80           ; Check interrupt pending bit (bit 7)
    beq check_uart     ; If not set, check next device

    ; Handle timer interrupt
    lda #$80
    sta $D001          ; Write to timer control register (acknowledge)

check_uart:
    ; Poll UART device
    lda $D100          ; Read UART status register
    and #$80           ; Check interrupt pending bit
    beq exit_isr       ; If not set, no more devices

    ; Handle UART interrupt
    lda $D102          ; Read UART data register (clears interrupt)

exit_isr:
    ; Restore accumulator
    pla

    ; Return from interrupt
    rti                ; Restores PC, status (including I flag)
```

---

## Timing Contract

### Interrupt Latency

**Maximum Latency**: 1 instruction + 7 cycles

**Breakdown**:
- **Worst case**: Current instruction completes (up to ~7 cycles for longest instruction)
- **IRQ check**: 0 cycles (part of instruction completion)
- **Service sequence**: 7 cycles (IRQ entry sequence)
- **Total**: ~14 cycles worst case from interrupt assertion to handler entry

**Guaranteed**:
- If IRQ line is active and I flag is clear, interrupt WILL be serviced after current instruction
- No interrupts are lost (level-sensitive line remains active until serviced)

### Cycle Accuracy

**Interrupt Entry**: Exactly 7 cycles
**Interrupt Exit (RTI)**: 6 cycles (pop status, pop PC low, pop PC high, update PC)

**Zero Overhead**: When IRQ line is inactive, no additional cycles consumed beyond normal instruction execution.

---

## Error Handling

### Invalid Vector

**Scenario**: IRQ vector points to 0x0000 or other invalid address

**Behavior**:
- CPU MUST still execute the 7-cycle sequence
- PC set to vector address (even if 0x0000)
- No error returned (matches real hardware)
- Result: CPU jumps to invalid address, likely executing invalid instruction

### Stack Overflow

**Scenario**: Stack pointer underflows during interrupt service (SP < 3)

**Behavior**:
- CPU SHOULD return `Err(CpuError::StackOverflow)`
- Interrupt service aborted
- PC and status NOT modified

### Device Acknowledgment Failure

**Scenario**: ISR reads device status but device doesn't clear interrupt flag

**Behavior**:
- IRQ line remains active after RTI
- CPU immediately re-enters ISR (infinite loop if device never clears)
- This is CORRECT behavior (matches real hardware)
- Device or ISR bug, not CPU bug

---

## Test Requirements

### Contract Verification Tests

1. **IRQ Line Query**:
   - Verify `irq_active()` returns false when no devices have interrupts
   - Verify `irq_active()` returns true when at least one device has interrupt
   - Verify multiple calls return consistent results

2. **Interrupt Service Sequence**:
   - Verify exactly 7 cycles consumed
   - Verify PC, status, and stack state after service
   - Verify I flag set after service
   - Verify handler address read from 0xFFFE-0xFFFF

3. **I Flag Respect**:
   - Verify interrupt NOT serviced when I flag set
   - Verify IRQ line can be active while I flag set (no error)

4. **ISR Execution**:
   - Verify ISR can read device status registers
   - Verify ISR can write device control registers
   - Verify RTI returns to correct address

5. **Multiple Devices**:
   - Verify IRQ line active if ANY device has interrupt
   - Verify IRQ line inactive only when ALL devices clear
   - Verify ISR can poll and acknowledge multiple devices

---

## Version History

**1.0.0** (2025-11-18):
- Initial contract definition
- Level-sensitive IRQ line model
- 7-cycle interrupt service sequence
- Memory-mapped device registers
