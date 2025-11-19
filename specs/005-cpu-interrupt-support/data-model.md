# Data Model: CPU Interrupt Support

**Date**: 2025-11-18
**Phase**: 1 (Design & Contracts)

## Overview

This document defines the data structures, state machines, and relationships for the interrupt support implementation. The model prioritizes simplicity and hardware fidelity over abstraction complexity.

## Core Entities

### 1. IRQ Line State

**Purpose**: Represents the shared interrupt request line state.

**Fields**:
```rust
// In CPU<M: MemoryBus> struct
pub struct CPU<M: MemoryBus> {
    // ... existing fields (a, x, y, sp, pc, flags, cycles, memory) ...

    /// IRQ line state: true when any device has unserviced interrupt
    /// Active LOW in hardware, but represented as active HIGH in code for clarity
    irq_pending: bool,
}
```

**Invariants**:
- `irq_pending` is true if and only if at least one device has an active interrupt request
- Checked after each instruction completes (at instruction boundary)
- Cleared only when all devices have cleared their interrupt requests

**Lifecycle**:
1. **Inactive** (false): No devices have pending interrupts
2. **Active** (true): One or more devices have pending interrupts
3. Transition **Inactive → Active**: Device asserts interrupt via memory-mapped register
4. Transition **Active → Inactive**: All devices clear their interrupt flags

---

### 2. Device Interrupt State

**Purpose**: Each device maintains its own interrupt request flag that contributes to the shared IRQ line.

**Pattern**: Devices implementing MemoryBus expose interrupt state through memory-mapped registers.

**Example Device Structure**:
```rust
/// Example interrupt-capable device
pub struct TimerDevice {
    /// Memory-mapped register base address
    base_address: u16,

    /// Internal interrupt request flag
    /// When true, this device contributes to CPU IRQ line
    interrupt_pending: bool,

    /// Other device-specific state
    counter: u16,
    // ...
}
```

**Memory-Mapped Register Layout** (example for timer device):
```text
Base + 0: STATUS register (read-only)
    Bit 7: Interrupt pending (1 = pending, 0 = none)
    Bit 6-0: Other status bits

Base + 1: CONTROL register (read-write)
    Bit 7: Interrupt enable (1 = enabled, 0 = disabled)
    Bit 6-0: Other control bits
    Write: Clear interrupt flag when bit pattern matches
```

**Invariants**:
- Device sets `interrupt_pending` when interrupt condition occurs
- Device clears `interrupt_pending` when ISR acknowledges (reads status or writes control)
- Device updates IRQ line state after each register access

**Lifecycle**:
1. **Idle**: No interrupt condition, `interrupt_pending = false`
2. **Asserted**: Interrupt condition occurs, `interrupt_pending = true`, contributes to IRQ line
3. **Acknowledged**: ISR reads status register, device aware ISR is handling it
4. **Cleared**: ISR writes control register or reads status, `interrupt_pending = false`, IRQ line contribution removed

---

### 3. Interrupt Sequence State

**Purpose**: Tracks the CPU's progress through the 7-cycle interrupt service routine entry.

**Implementation**: No explicit state machine needed. The interrupt sequence is a straight-line execution path triggered when IRQ conditions are met.

**Preconditions** (checked after instruction completes):
1. `irq_pending == true` (IRQ line active)
2. `flag_i == false` (I flag clear - interrupts enabled)
3. Not already in interrupt sequence (implicit - checked at instruction boundary)

**Sequence Operations** (7 cycles total):
```rust
// Conceptual pseudocode - actual implementation may vary
fn service_interrupt<M: MemoryBus>(&mut self) {
    // Cycle 1-3: Push PC and status to stack
    self.push_u16(self.pc);          // 2 cycles
    self.push_u8(self.status());     // 1 cycle

    // Cycle 4: Set interrupt disable flag
    self.flag_i = true;

    // Cycle 5-6: Read IRQ vector
    let vector_low = self.memory.read(0xFFFE);   // 1 cycle
    let vector_high = self.memory.read(0xFFFF);  // 1 cycle
    let handler_address = u16::from_le_bytes([vector_low, vector_high]);

    // Cycle 7: Jump to handler
    self.pc = handler_address;       // 1 cycle (internal operation)

    // Total: 7 cycles added to self.cycles
    self.cycles += 7;
}
```

**Postconditions**:
- PC points to interrupt handler entry point
- I flag is set (disabling nested interrupts)
- Stack contains return address and status register
- Cycle counter incremented by exactly 7

---

## Relationships

### CPU ↔ IRQ Line

```text
CPU:
  - Checks irq_pending after each instruction
  - Services interrupt if irq_pending && !flag_i
  - Does NOT modify irq_pending directly

IRQ Line:
  - Updated by devices via memory mapper
  - Reflects OR of all device interrupt_pending flags
```

### Devices ↔ IRQ Line

```text
Device:
  - Sets interrupt_pending when interrupt condition occurs
  - Notifies memory mapper to update CPU's irq_pending
  - Clears interrupt_pending when ISR acknowledges

Memory Mapper (or CPU bridge):
  - Aggregates all device interrupt_pending flags
  - Sets CPU's irq_pending if ANY device has interrupt pending
  - Clears CPU's irq_pending when ALL devices have no pending interrupts
```

### ISR ↔ Devices

```text
ISR (software running on CPU):
  - Polls device status registers (LDA $D000, etc.)
  - Identifies which devices have interrupts pending
  - Writes to device control registers to acknowledge
  - Returns via RTI instruction

Devices:
  - Respond to memory reads of status register
  - Respond to memory writes of control register
  - Clear interrupt_pending based on acknowledgment pattern
```

---

## State Transitions

### IRQ Line State Machine

```text
[No Interrupts] ──device asserts──→ [Interrupt Pending]
       ↑                                    │
       │                                    │
       └────────all devices clear───────────┘
```

### Interrupt Service State Machine

```text
[Normal Execution]
    │
    ├─→ Check IRQ after instruction
    │       │
    │       ├─→ irq_pending && !flag_i → [Service Interrupt]
    │       │                                 │
    │       │                                 ├─→ Push PC (2 cycles)
    │       │                                 ├─→ Push status (1 cycle)
    │       │                                 ├─→ Set I flag (0 cycles, part of push)
    │       │                                 ├─→ Read vector (2 cycles)
    │       │                                 └─→ Jump to handler (2 cycles)
    │       │                                      │
    │       │                                      v
    │       │                                 [ISR Execution]
    │       │                                      │
    │       │                                      ├─→ Poll devices
    │       │                                      ├─→ Handle interrupts
    │       │                                      └─→ RTI
    │       │                                           │
    │       └──────irq_pending || flag_i ─────────────┘
    │
    └─→ Continue normal execution
```

---

## Validation Rules

### CPU Invariants

1. **IRQ Check Timing**: CPU MUST check `irq_pending` after each instruction completes, before fetching the next instruction.

2. **I Flag Respect**: CPU MUST NOT service interrupts when `flag_i == true`, even if `irq_pending == true`.

3. **Cycle Accuracy**: Interrupt service routine entry MUST add exactly 7 cycles to the cycle counter.

4. **Stack Integrity**: Interrupt service MUST push PC (2 bytes) and status (1 byte) to stack before jumping to handler.

5. **Vector Reading**: Interrupt handler address MUST be read from memory addresses 0xFFFE (low byte) and 0xFFFF (high byte).

### Device Invariants

1. **Address Range Uniqueness**: Each device MUST declare a unique memory address range with no overlap with other devices.

2. **Register Determinism**: Reading from the same device register twice MUST return consistent results unless device state changed between reads.

3. **Interrupt Clearing**: Device MUST provide a mechanism for ISR to clear `interrupt_pending` flag (read status, write control, or other defined pattern).

4. **IRQ Contribution**: Device MUST accurately reflect its `interrupt_pending` state in its contribution to the shared IRQ line.

### System Invariants

1. **IRQ Line Consistency**: CPU's `irq_pending` flag MUST be true if and only if at least one device has `interrupt_pending == true`.

2. **No Lost Interrupts**: If a device asserts an interrupt and the I flag is clear, the CPU MUST eventually service the interrupt (no interrupts lost).

3. **No Spurious Interrupts**: CPU MUST NOT service interrupts when `irq_pending == false`.

---

## Example Scenarios

### Scenario 1: Single Device Interrupt

```text
1. Timer device triggers interrupt
   - Timer sets interrupt_pending = true
   - Timer updates IRQ line via memory mapper
   - CPU's irq_pending becomes true

2. CPU completes current instruction
   - Checks irq_pending → true
   - Checks flag_i → false
   - Enters interrupt service sequence (7 cycles)

3. ISR executes
   - Reads timer status register at $D000
   - Timer sees status read, clears interrupt_pending
   - Timer updates IRQ line, CPU's irq_pending becomes false
   - ISR performs timer handling logic
   - ISR executes RTI

4. CPU returns to normal execution
```

### Scenario 2: Multiple Devices Simultaneous

```text
1. Timer and UART both trigger interrupts
   - Timer sets interrupt_pending = true
   - UART sets interrupt_pending = true
   - CPU's irq_pending becomes true (OR of both)

2. CPU enters interrupt service (7 cycles)

3. ISR executes
   - Reads timer status → timer has interrupt
   - Reads UART status → UART has interrupt
   - Writes to timer control → timer clears interrupt_pending
   - IRQ line still active (UART still pending)

4. ISR continues
   - Writes to UART control → UART clears interrupt_pending
   - IRQ line becomes inactive (all devices cleared)
   - ISR executes RTI

5. CPU returns to normal execution
```

### Scenario 3: Nested Interrupt Attempt

```text
1. First interrupt triggers
   - CPU services interrupt (7 cycles)
   - I flag set to true during service

2. Second device triggers interrupt during ISR
   - Device sets interrupt_pending = true
   - CPU's irq_pending becomes true

3. CPU continues ISR execution
   - Checks irq_pending after each instruction → true
   - Checks flag_i → true (set during interrupt service)
   - Does NOT service second interrupt (I flag prevents)

4. ISR executes RTI
   - Restores original status register (I flag cleared)
   - Returns to interrupted code

5. CPU checks interrupts immediately
   - irq_pending still true (second device not cleared)
   - flag_i now false (restored by RTI)
   - Services second interrupt (7 cycles again)
```

---

## Implementation Notes

### Memory Mapper Integration

The CPU doesn't directly manage device interrupt states. Instead, a memory mapper (or device collection wrapper) aggregates device interrupt flags:

```rust
pub struct MemoryMapper {
    devices: Vec<Box<dyn InterruptDevice>>,
    // ...
}

impl MemoryMapper {
    /// Check if any device has pending interrupt
    pub fn irq_line_active(&self) -> bool {
        self.devices.iter().any(|dev| dev.has_interrupt())
    }
}
```

The CPU queries this during instruction execution:

```rust
impl<M: MemoryBus> CPU<M> {
    pub fn step(&mut self) -> Result<(), CpuError> {
        // Execute instruction...

        // Check interrupts at instruction boundary
        self.irq_pending = self.memory.irq_line_active();

        if self.irq_pending && !self.flag_i {
            self.service_interrupt()?;
        }

        Ok(())
    }
}
```

### Alternative: IRQ Line Callback

If MemoryBus can't return IRQ state, use a callback pattern:

```rust
pub trait MemoryBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
    fn irq_active(&self) -> bool;  // Check IRQ line state
}
```

---

## Testing Strategy

### Unit Tests

1. **IRQ Line State**: Verify CPU correctly reads IRQ line state
2. **Interrupt Service**: Verify 7-cycle sequence with correct stack pushes
3. **I Flag Respect**: Verify interrupts blocked when I flag set
4. **Vector Reading**: Verify handler address read from 0xFFFE-0xFFFF

### Integration Tests

1. **Single Device**: Full interrupt cycle with one device
2. **Multiple Devices**: ISR polls multiple devices, clears all
3. **Nested Attempts**: Second interrupt during ISR (should be blocked)
4. **Rapid Interrupts**: High interrupt rate doesn't lose or duplicate interrupts

### Cycle Counting Tests

1. **7-Cycle Sequence**: Verify cycle counter increments by exactly 7
2. **Overhead Measurement**: Verify zero overhead when no interrupts pending
