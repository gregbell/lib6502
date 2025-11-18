# Research: CPU Interrupt Support

**Date**: 2025-11-18
**Phase**: 0 (Research & Unknowns)

## Overview

This document consolidates research findings and design decisions made during the specification clarification phase. All technical unknowns were resolved through analysis of real 6502 hardware behavior documented in the MOS 6502 Programming Manual.

## Research Questions & Findings

### 1. Interrupt Mechanism Architecture

**Question**: Should the interrupt mechanism mimic real 6502 hardware behavior (level-sensitive IRQ line, no queueing) or implement a modern queued interrupt controller?

**Decision**: Level-sensitive IRQ line matching real 6502 hardware

**Rationale**:
- **Hardware fidelity**: The 6502 has a single level-sensitive IRQ pin (active low) shared by all devices via wire-OR. No built-in queue or priority mechanism exists.
- **Cycle accuracy**: Matching hardware behavior ensures assembly code behaves identically to physical hardware.
- **Simplicity**: A single boolean flag (IRQ line active/inactive) is simpler than managing an interrupt queue.
- **ISR compatibility**: Real 6502 ISRs expect to poll device status registers to identify interrupt sources. A queue would require fundamentally different ISR patterns.

**Alternatives Considered**:
- **Queued model**: Maintain FIFO queue of interrupt events, each device signal creates queue entry. **Rejected** because it doesn't match hardware behavior and requires ISRs to be written differently than on real hardware.
- **Hybrid model**: Level-sensitive IRQ line but track pending devices in implementation-internal queue. **Rejected** as unnecessary complexity - ISR polling is the correct hardware pattern.

**Implementation Notes**:
- CPU struct contains single `irq_line_active: bool` field
- Devices can independently assert/clear their contribution to the IRQ line
- Multiple devices pulling IRQ low simultaneously keeps line active until all clear
- No interrupt "events" or "signals" to track - only current line state matters

---

### 2. Device Notification Model

**Question**: How should devices be notified that their interrupt is being serviced?

**Decision**: ISR acknowledges explicitly via memory-mapped register access

**Rationale**:
- **Hardware accuracy**: Real 6502 hardware has no automatic notification mechanism. The ISR must explicitly read from or write to device registers to acknowledge interrupts.
- **Visible acknowledgment**: Memory read/write operations are visible in code, making ISR behavior explicit and debuggable.
- **Device autonomy**: Devices control when to clear their interrupt flag based on specific acknowledgment patterns (read status register, write to control register, etc.).
- **No hidden coupling**: CPU doesn't need to know which devices exist or how to notify them.

**Alternatives Considered**:
- **Automatic notification**: CPU automatically notifies all devices with pending interrupts when entering ISR. **Rejected** because it doesn't match hardware behavior and adds hidden coupling between CPU and devices.
- **Callback registration**: Devices register callback functions invoked during ISR. **Rejected** as non-hardware-like and adds runtime overhead + complexity.

**Implementation Notes**:
- CPU has no device references or notification mechanism
- ISR code reads device status registers (memory addresses) to identify interrupt sources
- ISR writes to device control registers (memory addresses) to clear interrupt flags
- Devices clear their IRQ line contribution when appropriate register accessed

---

### 3. Device Register Exposure

**Question**: How should devices expose their interrupt status for ISR polling?

**Decision**: Memory-mapped status and control registers via MemoryBus trait

**Rationale**:
- **Hardware pattern**: Real 6502 devices expose status/control via memory-mapped I/O. ISR polls by reading memory addresses.
- **Existing abstraction**: The MemoryBus trait already provides the read/write interface needed. No new abstractions required.
- **Language independence**: Any code implementing MemoryBus can create an interrupt-capable device, including JavaScript via WASM bindings.
- **Testability**: Memory-mapped registers are easy to test - just read/write memory and verify behavior.

**Alternatives Considered**:
- **Direct API calls**: ISR calls device methods like `device.has_interrupt()`, `device.clear_interrupt()`. **Rejected** because it requires CPU to have direct device references (violates modularity) and doesn't match hardware patterns.
- **Callback registration**: Devices register callback functions for status queries. **Rejected** due to runtime overhead and non-hardware-like design.

**Implementation Notes**:
- Devices implement MemoryBus trait (or integrate with memory mapper)
- Status register: Read returns interrupt status + other device state
- Control register: Write clears interrupt flag and performs control actions
- ISR code uses normal LDA/STA instructions to interact with devices

---

### 4. Memory Address Allocation

**Question**: How should memory addresses be allocated for device status and control registers?

**Decision**: Device specifies address range at construction time

**Rationale**:
- **Flexibility**: Matches how real hardware is configured - via jumpers, configuration registers, or memory mapping hardware.
- **Explicit configuration**: System integrator explicitly declares device memory layout, making it visible and debuggable.
- **Validation support**: System can validate address ranges don't overlap before devices are used.
- **ISR predictability**: ISR code knows exact addresses to poll (typically documented in device spec).

**Alternatives Considered**:
- **Fixed address ranges**: System reserves specific ranges for device types (e.g., 0xD000-0xD0FF for UART). **Rejected** as inflexible - can't have multiple instances of same device type or customize layout.
- **Dynamic allocation**: System automatically assigns addresses from available pool. **Rejected** because ISR code wouldn't know where to find devices without runtime lookup mechanism.

**Implementation Notes**:
- Device constructors accept address range parameters
- System validates ranges don't overlap (compile-time or init-time check)
- ISR code uses hardcoded addresses matching device construction
- Memory mapper routes reads/writes to appropriate device

---

### 5. Interrupt Cycle Cost

**Question**: What is the cycle cost for the interrupt processing sequence?

**Decision**: 7 cycles

**Rationale**:
- **Hardware specification**: MOS 6502 Programming Manual documents IRQ sequence as consuming 7 cycles.
- **Cycle breakdown** (per 6502 spec):
  1. Finish current instruction
  2. Push PCH (high byte) to stack - 1 cycle
  3. Push PCL (low byte) to stack - 1 cycle
  4. Push status register to stack - 1 cycle
  5. Read IRQ vector low byte from 0xFFFE - 1 cycle
  6. Read IRQ vector high byte from 0xFFFF - 1 cycle
  7. Set I flag (interrupt disable) - included in cycle 5
  8. Jump to handler - 2 cycles (internal operation + PC update)

  Total: 7 cycles (internal cycle bookkeeping may vary by implementation detail, but total is 7)

- **Test compatibility**: Existing 6502 test suites (Klaus Dormann functional test) expect 7-cycle interrupt cost.

**Alternatives Considered**:
- **Different cycle counts**: None considered - 6502 hardware behavior is well-documented and non-negotiable for accuracy.

**Implementation Notes**:
- Interrupt sequence implementation adds exactly 7 to cycle counter
- Each sub-operation (stack push, vector read) may internally track cycles, but total must be 7
- Test cases verify cycle counter increments by exactly 7 during interrupt

---

## Best Practices Applied

### Rust Memory Safety in Interrupt Context

**Pattern**: Shared mutable state (IRQ line) accessed without interior mutability

**Implementation**:
- IRQ line state managed by CPU, modified during instruction execution
- No concurrent access (single-threaded emulation)
- No RefCell/Mutex needed (deterministic, sequential execution)

### WASM Compatibility

**Constraints**:
- No callbacks (WASM function pointers are complex)
- No dynamic dispatch where avoidable
- All state in plain structs (serializable, debuggable)

**Solution**:
- Memory-mapped I/O avoids callbacks entirely
- IRQ line is simple bool flag
- Device state accessed via MemoryBus reads/writes (trait objects are acceptable)

### Testing Strategy

**Approach**:
1. **Unit tests**: Test interrupt checking logic in isolation (mock memory, fake devices)
2. **Integration tests**: Test full interrupt scenarios with real device implementations
3. **Cycle counting tests**: Verify exactly 7 cycles consumed by interrupt sequence
4. **ISR tests**: Verify ISR can poll and acknowledge multiple devices

---

## References

- **MOS 6502 Programming Manual**: Interrupt timing and behavior specifications
- **Klaus Dormann 6502 Functional Test**: Reference test suite for validation
- **Project Constitution**: Principles guiding design decisions (modularity, WASM portability, cycle accuracy)

---

## Remaining Work

All research questions resolved. No unknowns remain for implementation. Ready to proceed to Phase 1 (Design & Contracts).
