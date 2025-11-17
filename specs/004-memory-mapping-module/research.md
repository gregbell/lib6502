# Research: Memory Mapping Module with UART Device Support

**Feature**: 004-memory-mapping-module
**Date**: 2025-11-17
**Status**: Complete

## Overview

This document consolidates research findings for implementing a memory mapping system with 6551 UART device support. All technical decisions have been made and documented below.

## 1. 6551 UART Hardware Specification

### Decision: W65C51N ACIA Register Layout

**Chosen Approach**: Implement four memory-mapped registers at base address + offset:
- **+0**: Data register (read: receive, write: transmit)
- **+1**: Status register (read-only)
- **+2**: Command register (read/write)
- **+3**: Control register (read/write)

**Rationale**: This matches the W65C51N ACIA chip used in Ben Eater's breadboard computer and vintage 6502 systems. Well-documented, simple to implement, widely understood by hobbyist community.

**Alternatives Considered**:
- 16550 UART (more complex, FIFO buffers, overkill for initial implementation)
- Custom minimal UART (less educational value, harder to port existing 6502 software)

### Register Bit Definitions

**Status Register (read-only)**:
```
Bit 7: Interrupt (IRQ) flag
Bit 6: Data Set Ready (DSR) - not implemented (always 0)
Bit 5: Data Carrier Detect (DCD) - not implemented (always 0)
Bit 4: Transmitter Data Register Empty (TDRE)
Bit 3: Receiver Data Register Full (RDRF)
Bit 2: Overrun error
Bit 1: Framing error - not implemented (always 0)
Bit 0: Parity error - not implemented (always 0)
```

**Command Register (read/write)**:
```
Bit 7-5: Parity mode (000=disabled, others not initially implemented)
Bit 4: Parity type (0=even, 1=odd) - not initially implemented
Bit 3: Receiver echo mode (1=enabled)
Bit 2: Transmitter interrupt enable - not initially implemented
Bit 1: Receiver interrupt enable - not initially implemented
Bit 0: Data Terminal Ready (DTR) - not implemented
```

**Control Register (read/write)**:
```
Bit 7: Stop bits (0=1 stop bit, 1=2 stop bits) - stored but not enforced
Bit 6-5: Word length (00=8 bits, others not implemented)
Bit 4: Receiver clock source - not implemented
Bit 3-0: Baud rate (0000=external, others stored but not enforced)
```

**Decision**: Implement status/command/control registers with subset of features. Store configuration values but don't enforce timing (baud rate, stop bits, parity) in initial version. Focus on data flow correctness.

**Rationale**: Emulation doesn't need real-time baud rate enforcement. Storing values allows 6502 code to configure UART and read back settings (important for software compatibility). Actual byte transmission is immediate via callback.

## 2. Memory Mapping Patterns

### Decision: Linear Search with Range Checks

**Chosen Approach**: Store devices in `Vec<DeviceMapping>` where each mapping contains base address and boxed device. On read/write, iterate through mappings to find device whose range contains target address.

```rust
struct DeviceMapping {
    base_addr: u16,
    device: Box<dyn Device>,
}

impl MappedMemory {
    fn find_device(&mut self, addr: u16) -> Option<(&mut Box<dyn Device>, u16)> {
        for mapping in &mut self.devices {
            let size = mapping.device.size();
            if addr >= mapping.base_addr && addr < mapping.base_addr + size {
                let offset = addr - mapping.base_addr;
                return Some((&mut mapping.device, offset));
            }
        }
        None
    }
}
```

**Rationale**: Simple, correct, easy to understand. For typical 6502 systems with 3-10 devices, linear search is fast enough (<100ns). No complex indexing structures needed.

**Alternatives Considered**:
- Interval tree (overcomplicated, harder to maintain)
- HashMap with page-level granularity (memory overhead, complexity)
- Sorted array with binary search (marginal performance gain, harder to insert/remove)

### Unmapped Address Behavior

**Decision**: Return `0xFF` for unmapped reads, ignore unmapped writes.

**Rationale**: Common 6502 hardware behavior (floating bus pulls high). Predictable for debugging (not random garbage). Matches NES, Apple II, other systems.

**Alternatives Considered**:
- Return `0x00` (less common, but simpler)
- Return last value on data bus (more realistic but adds state complexity)
- Panic (violates constitution, breaks WASM portability)

### Overlapping Address Ranges

**Decision**: Disallow overlapping ranges at registration time. Return error if new device overlaps existing.

**Rationale**: Overlapping ranges indicate configuration error. Better to fail fast than have undefined priority behavior.

**Alternatives Considered**:
- Priority-based resolution (first wins, last wins, explicit priority field) - adds complexity
- Allow overlap and document "undefined behavior" - poor developer experience

## 3. Rust Trait Design for Devices

### Decision: Simple Device Trait with Offset-Based Addressing

**Chosen Approach**:
```rust
pub trait Device {
    /// Read byte from device at offset relative to device base
    fn read(&self, offset: u16) -> u8;

    /// Write byte to device at offset relative to device base
    fn write(&mut self, offset: u16, value: u8);

    /// Return size of device's address space
    fn size(&self) -> u16;
}
```

**Rationale**:
- **Offset-based**: Device doesn't need to know its mapped address (reusable)
- **Immutable read**: Matches `MemoryBus` trait, allows read-only status registers
- **Mutable write**: Explicit side effects (buffer updates, flag changes)
- **Size method**: Mapper needs to know device bounds for range checks

**Alternatives Considered**:
- Absolute addressing (device gets full 16-bit address) - couples device to mapping location
- Separate ReadDevice/WriteDevice traits - overcomplicated, most devices do both
- Add `reset()` method - useful but not needed for MVP, can add later

### Error Handling Pattern

**Decision**: No panics, no Result types. Invalid operations return garbage or are silently ignored.

**Rationale**: Matches 6502 hardware behavior. Satisfies constitution (no panics). Simplifies API (no error handling boilerplate).

**Example**: Reading from write-only register returns last internal value. Writing to read-only register is ignored.

### Testing Strategy

**Decision**: Devices expose public methods for state inspection (not just trait methods).

**Example**:
```rust
impl Uart6551 {
    pub fn status(&self) -> u8 { ... }           // For testing
    pub fn rx_buffer_len(&self) -> usize { ... } // For verification
}
```

**Rationale**: Unit tests need to verify internal state. Public inspection methods better than `#[cfg(test)] pub` hacks.

## 4. WASM Callback Interface

### Decision: Optional Boxed Fn Trait for Transmit Callback

**Chosen Approach**:
```rust
pub struct Uart6551 {
    on_transmit: Option<Box<dyn Fn(u8)>>,
}

impl Uart6551 {
    pub fn set_transmit_callback<F>(&mut self, callback: F)
    where
        F: Fn(u8) + 'static,
    {
        self.on_transmit = Some(Box::new(callback));
    }
}
```

**Rationale**:
- **Option**: Not all uses need callbacks (testing, headless execution)
- **Fn not FnMut**: Callback shouldn't mutate captured state (simpler ownership)
- **Box**: Avoids generic type parameter on struct (simpler API, allows trait objects)
- **'static**: WASM callbacks must outlive device (no borrowed references across boundary)

**Alternatives Considered**:
- Function pointer (`Option<fn(u8)>`) - can't capture environment, less flexible
- Generic type parameter - complicates API, harder to store in collections
- FnMut callback - allows mutation but complicates WASM binding (mutable borrows)

### Browser Integration Pattern (xterm.js)

**Decision**: Unidirectional callbacks, pull-based receive.

**Flow**:
1. **Transmit**: UART calls `on_transmit(byte)` → WASM binding → JS → `term.write(String.fromCharCode(byte))`
2. **Receive**: JS `term.onData(data)` → WASM binding → calls `uart.receive_byte(byte)` → UART buffers

**Rationale**: Simple, no shared state, no threading. Transmit is push (UART initiates), receive is push from JS (terminal initiates). Clean separation.

**Example WASM binding**:
```rust
#[wasm_bindgen]
pub fn uart_set_callback(callback: js_sys::Function) {
    uart.set_transmit_callback(move |byte| {
        callback.call1(&JsValue::NULL, &JsValue::from(byte)).unwrap();
    });
}

#[wasm_bindgen]
pub fn uart_receive(byte: u8) {
    uart.receive_byte(byte);
}
```

### Data Ownership and Buffering

**Decision**: UART owns buffers internally (`VecDeque<u8>`). Default 256 bytes, configurable.

**Rationale**:
- **VecDeque**: Efficient FIFO (O(1) push/pop on both ends)
- **Internal ownership**: No shared pointers, clear lifetime
- **Fixed size**: Prevents unbounded growth, matches hardware buffers
- **Overflow flag**: Set bit 2 of status register when buffer full

**Buffer behavior**:
- **RX buffer full**: New bytes dropped, overflow flag set
- **TX buffer full**: Not implemented initially (transmit is immediate via callback)

## 5. Additional Considerations

### Cycle Accuracy for Memory-Mapped I/O

**Decision**: UART reads/writes use same cycle cost as memory (per existing MemoryBus contract). No additional cycles for register access.

**Rationale**: Simplicity. Real 6551 has same cycle timing as memory reads (devices on address/data bus). Can add device-specific cycle costs later if needed for accuracy.

### Device Initialization and Reset

**Decision**: Devices provide `new()` constructor with sane defaults. No explicit reset mechanism in initial version.

**Rationale**: 6502 programs typically initialize hardware by writing registers. Explicit reset can be added later if needed (e.g., for 6502 RESET vector handling).

### Debugging and Logging

**Decision**: No built-in logging in core library. Consumers can wrap devices with logging decorator.

**Rationale**: Maintains zero-dependency requirement. Advanced users can implement Device trait as wrapper that logs then delegates.

**Example**:
```rust
struct LoggingDevice<D: Device> {
    inner: D,
}

impl<D: Device> Device for LoggingDevice<D> {
    fn read(&self, offset: u16) -> u8 {
        let value = self.inner.read(offset);
        eprintln!("READ  {:04X} -> {:02X}", offset, value);
        value
    }
    // ...
}
```

## Summary of Key Decisions

| Area | Decision | Rationale |
|------|----------|-----------|
| Register layout | W65C51N ACIA (base+0/1/2/3) | Standard, educational, compatible |
| Timing enforcement | Store config, don't enforce baud/parity | Emulation doesn't need real-time timing |
| Device lookup | Linear search Vec | Simple, fast enough for 3-10 devices |
| Unmapped reads | Return 0xFF | Common hardware behavior, predictable |
| Address overlap | Disallow, return error | Fail fast on configuration errors |
| Device trait | Offset-based read/write/size | Reusable, matches MemoryBus pattern |
| Error handling | No panics, no Results | Matches hardware, satisfies constitution |
| Callback type | Option<Box<dyn Fn(u8)>> | WASM-compatible, flexible, simple |
| Receive pattern | Pull-based (external calls receive_byte) | Clean separation, no shared state |
| Buffers | VecDeque, 256 bytes default | Efficient FIFO, bounded growth |
| Cycle costs | Same as memory | Simple, can enhance later |

## References

- W65C51N ACIA Datasheet: https://www.westerndesigncenter.com/wdc/documentation/w65c51n.pdf
- Ben Eater 6551 UART video: https://www.youtube.com/watch?v=zsERDRM1oy8
- Rust WASM book on callbacks: https://rustwasm.github.io/wasm-bindgen/reference/receiving-js-closures-in-rust.html
- xterm.js API documentation: https://xtermjs.org/docs/api/terminal/classes/terminal/

## Open Questions

**Resolved**: All research questions answered. No blockers for implementation.

**Future Enhancements** (out of scope for MVP):
- Hardware flow control (RTS/CTS)
- Interrupt-driven I/O (IRQ on receive/transmit)
- DMA-style bulk transfers
- Multiple UART instances
- Configurable buffer sizes per instance
