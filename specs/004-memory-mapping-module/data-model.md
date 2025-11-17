# Data Model: Memory Mapping Module with UART Device Support

**Feature**: 004-memory-mapping-module
**Date**: 2025-11-17
**Status**: Complete

## Overview

This document defines the core entities, their relationships, state transitions, and validation rules for the memory mapping system.

## Core Entities

### 1. Device (Trait)

**Purpose**: Abstract interface for any memory-mapped hardware component.

**Attributes**:
- None (trait defines behavior only)

**Behavior**:
- `read(offset: u16) -> u8`: Read byte at offset within device's address space
- `write(offset: u16, value: u8)`: Write byte at offset within device's address space
- `size() -> u16`: Return size of device's address space in bytes

**Relationships**:
- Implemented by: RamDevice, RomDevice, Uart6551
- Used by: MappedMemory (via Box<dyn Device>)

**Validation Rules**:
- `read()` must never panic
- `write()` must never panic
- `size()` must return constant value (cannot change after construction)
- `offset` parameter is always `< size()` (enforced by mapper)

**State**: Stateless trait, implementations may have internal state

---

### 2. MappedMemory (Struct)

**Purpose**: Routes memory read/write operations to registered devices based on address ranges.

**Attributes**:
```rust
struct MappedMemory {
    devices: Vec<DeviceMapping>,
    unmapped_value: u8,  // Default: 0xFF
}

struct DeviceMapping {
    base_addr: u16,
    device: Box<dyn Device>,
}
```

**Behavior**:
- Implements `MemoryBus` trait (read/write)
- `add_device(base_addr: u16, device: Box<dyn Device>) -> Result<(), OverlapError>`
- `remove_device(base_addr: u16) -> Option<Box<dyn Device>>`

**Relationships**:
- Contains: Multiple DeviceMapping entries
- Each DeviceMapping contains: One Device instance
- Implements: MemoryBus (used by CPU)

**Validation Rules**:
- Device address ranges must not overlap
- base_addr + device.size() must not exceed 0x10000 (64KB)
- Devices cannot be added while borrowed by CPU (enforced by Rust)

**State Transitions**:
```
Empty -> HasDevices (via add_device)
HasDevices -> HasDevices (via add_device, if no overlap)
HasDevices -> Empty (via remove_device, if last device removed)
```

**Invariants**:
- No overlapping address ranges in `devices` vector
- `devices` can be empty (all reads return `unmapped_value`)

---

### 3. RamDevice (Struct)

**Purpose**: Provides readable and writable memory storage.

**Attributes**:
```rust
pub struct RamDevice {
    data: Vec<u8>,
}
```

**Behavior**:
- Implements `Device` trait
- `new(size: u16) -> Self`: Create RAM with specified size
- `load_bytes(&mut self, offset: u16, bytes: &[u8])`: Initialize RAM contents

**Relationships**:
- Implements: Device
- Used by: MappedMemory

**Validation Rules**:
- Size must be > 0 and ≤ 65536
- Read/write offsets must be < size (enforced by caller)

**State Transitions**:
```
Initialized (zeros) -> Written (via write() or load_bytes())
Written -> Written (via subsequent writes)
```

**Invariants**:
- `data.len()` equals size specified at construction

---

### 4. RomDevice (Struct)

**Purpose**: Provides read-only memory storage (writes ignored).

**Attributes**:
```rust
pub struct RomDevice {
    data: Vec<u8>,
}
```

**Behavior**:
- Implements `Device` trait
- `new(data: Vec<u8>) -> Self`: Create ROM with initial contents
- Writes are silently ignored (no-op)

**Relationships**:
- Implements: Device
- Used by: MappedMemory

**Validation Rules**:
- Data length must be > 0 and ≤ 65536
- Read offsets must be < size (enforced by caller)

**State Transitions**:
- Immutable (data never changes after construction)

**Invariants**:
- `data` contents never modified after construction

---

### 5. Uart6551 (Struct)

**Purpose**: Emulates W65C51N ACIA serial communication device.

**Attributes**:
```rust
pub struct Uart6551 {
    // Registers
    data_register: u8,
    status_register: u8,
    command_register: u8,
    control_register: u8,

    // Buffers
    rx_buffer: VecDeque<u8>,
    tx_buffer: VecDeque<u8>,  // Future: currently unused (immediate transmit)

    // Configuration
    rx_buffer_capacity: usize,  // Default: 256

    // Callbacks
    on_transmit: Option<Box<dyn Fn(u8)>>,
}
```

**Behavior**:
- Implements `Device` trait
- `new() -> Self`: Create UART with default settings
- `set_transmit_callback<F: Fn(u8) + 'static>(&mut self, callback: F)`
- `receive_byte(&mut self, byte: u8)`: Queue byte in RX buffer
- `status(&self) -> u8`: Get status register (for testing)
- `rx_buffer_len(&self) -> usize`: Get RX buffer count (for testing)

**Relationships**:
- Implements: Device
- Used by: MappedMemory
- Calls: on_transmit callback (when byte written to data register)
- Called by: External code (receive_byte when terminal sends data)

**Validation Rules**:
- `size()` always returns 4 (registers at offset 0/1/2/3)
- Status register (offset 1) is read-only
- Invalid offset (>3) reads return 0x00, writes are ignored
- RX buffer overflow sets bit 2 of status register

**State Transitions**:

**Receive Path**:
```
Idle -> Receiving (receive_byte called)
Receiving -> BufferFull (rx_buffer.len() == capacity)
BufferFull -> Overflow (receive_byte called, byte dropped)
[Any state] -> Idle (6502 reads data register, clears RDRF flag)
```

**Transmit Path**:
```
Idle -> Transmitting (6502 writes data register)
Transmitting -> Idle (callback invoked, TDRE flag set)
```

**Invariants**:
- `rx_buffer.len()` ≤ `rx_buffer_capacity`
- Status bit 4 (TDRE) is always 1 (transmitter always ready)
- Status bit 3 (RDRF) is 1 iff `!rx_buffer.is_empty()`
- Status bit 2 (overrun) is sticky (cleared only by reading status then data)

---

## Register Mappings

### UART Registers (offset from base address)

| Offset | Name | Read | Write | Description |
|--------|------|------|-------|-------------|
| 0x0 | Data | RX byte (from buffer) | TX byte (to callback) | Receive/transmit data |
| 0x1 | Status | Status flags | Ignored | TDRE, RDRF, overrun, IRQ |
| 0x2 | Command | Command config | Command config | Parity, echo, interrupts |
| 0x3 | Control | Control config | Control config | Baud rate, word length, stop bits |

### Status Register Bit Definitions

| Bit | Name | Meaning (when set) |
|-----|------|-------------------|
| 7 | IRQ | Interrupt request (not implemented, always 0) |
| 6 | DSR | Data Set Ready (not implemented, always 0) |
| 5 | DCD | Data Carrier Detect (not implemented, always 0) |
| 4 | TDRE | Transmitter Data Register Empty (always 1) |
| 3 | RDRF | Receiver Data Register Full (`!rx_buffer.is_empty()`) |
| 2 | Overrun | Receive buffer overflow occurred |
| 1 | Framing Error | (not implemented, always 0) |
| 0 | Parity Error | (not implemented, always 0) |

### Command Register Bit Definitions

| Bit | Name | Meaning |
|-----|------|---------|
| 7-5 | Parity Mode | 000=disabled (other modes not implemented) |
| 4 | Parity Type | 0=even, 1=odd (not enforced) |
| 3 | Echo Mode | 1=echo received bytes to transmit |
| 2 | TX IRQ Enable | (not implemented) |
| 1 | RX IRQ Enable | (not implemented) |
| 0 | DTR | Data Terminal Ready (not implemented) |

### Control Register Bit Definitions

| Bit | Name | Meaning |
|-----|------|---------|
| 7 | Stop Bits | 0=1 stop bit, 1=2 stop bits (stored, not enforced) |
| 6-5 | Word Length | 00=8 bits, other values not implemented |
| 4 | Clock Source | (not implemented) |
| 3-0 | Baud Rate | 0000=external clock, others stored but not enforced |

---

## State Transition Diagrams

### UART Receive Buffer State

```
          receive_byte()
              (buffer not full)
    ┌─────────────────────────────┐
    │                             │
    ▼                             │
┌────────┐  rx full   ┌───────────┴────┐  rx overflow  ┌──────────┐
│  IDLE  │───────────>│   RECEIVING    │──────────────>│ OVERFLOW │
│ RDRF=0 │            │   RDRF=1       │               │ RDRF=1   │
└────────┘            │   Overrun=0    │               │ Overrun=1│
    ▲                 └───────────┬────┘               └──────────┘
    │                             │                         │
    │  6502 reads data register   │                         │
    │  (pops from buffer)         │                         │
    └─────────────────────────────┴─────────────────────────┘
```

### UART Transmit Path

```
┌────────────┐   6502 writes data register   ┌──────────────┐
│   IDLE     │──────────────────────────────>│ TRANSMITTING │
│  TDRE=1    │                                │   TDRE=1*    │
└────────────┘                                └──────┬───────┘
     ▲                                               │
     │                                               │
     │         on_transmit callback invoked          │
     └───────────────────────────────────────────────┘

* TDRE always 1 (transmitter always ready, no buffering)
```

---

## Relationships Diagram

```
┌─────────┐
│   CPU   │ (uses MemoryBus trait)
└────┬────┘
     │
     ▼
┌────────────────┐
│ MappedMemory   │ (implements MemoryBus)
│                │
│ - devices: Vec │
│ - unmapped: u8 │
└────┬───────────┘
     │
     ├───> DeviceMapping { base_addr, device: Box<dyn Device> }
     │
     ├───> DeviceMapping { base_addr, device: Box<dyn Device> }
     │
     └───> DeviceMapping { base_addr, device: Box<dyn Device> }
                               │
                ┌──────────────┼──────────────┐
                ▼              ▼              ▼
           ┌─────────┐   ┌─────────┐   ┌──────────┐
           │RamDevice│   │RomDevice│   │Uart6551  │
           └─────────┘   └─────────┘   └────┬─────┘
                                             │
                                             ├──> on_transmit: Fn(u8)
                                             │         │
                                             │         ▼
                                             │    [Browser Terminal]
                                             │
                                             ▼
                                        rx_buffer: VecDeque<u8>
                                             ▲
                                             │
                                        [Browser Terminal]
                                     (via receive_byte())
```

---

## Validation Rules Summary

### Address Mapping Validation

1. **No overlap**: `∀ devices d1, d2: d1.range ∩ d2.range = ∅`
2. **Within 64KB**: `base_addr + device.size() ≤ 0x10000`
3. **Non-zero size**: `device.size() > 0`

### UART Buffer Validation

1. **Buffer capacity**: `rx_buffer.len() ≤ rx_buffer_capacity`
2. **Status consistency**: `RDRF bit = !rx_buffer.is_empty()`
3. **Overrun sticky**: Once set, cleared only by read status → read data sequence
4. **Transmitter ready**: `TDRE bit = 1` (always)

### Device Operation Validation

1. **No panics**: All operations must succeed or fail gracefully
2. **Offset bounds**: Caller ensures `offset < device.size()`
3. **Read determinism**: Multiple reads at same offset return same value (unless device state changes)
4. **Write idempotence**: Writing same value multiple times has same effect as writing once

---

## Extensibility

### Adding New Device Types

To add a new device (e.g., graphics chip, timer):

1. Implement `Device` trait:
   ```rust
   impl Device for MyDevice {
       fn read(&self, offset: u16) -> u8 { /* ... */ }
       fn write(&mut self, offset: u16, value: u8) { /* ... */ }
       fn size(&self) -> u16 { /* address space size */ }
   }
   ```

2. Register with MappedMemory:
   ```rust
   memory.add_device(0x6000, Box::new(MyDevice::new()))?;
   ```

3. Device can use same patterns as UART:
   - Internal state (registers, buffers)
   - Callbacks for external interaction
   - Public inspection methods for testing

### Future Enhancements

Entities that may be added later:

- **InterruptController**: Manages IRQ/NMI signals from devices
- **DmaController**: Direct memory access for bulk transfers
- **TimerDevice**: Periodic interrupts, cycle counting
- **VideoDevice**: Frame buffer, sprite registers
- **AudioDevice**: Waveform generators, sample playback

All follow same pattern: implement Device trait, register with MappedMemory.
