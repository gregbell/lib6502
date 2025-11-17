# Device Trait Contract

**Feature**: 004-memory-mapping-module
**Date**: 2025-11-17
**Version**: 1.0.0

## Overview

The `Device` trait defines the interface for all memory-mapped hardware components in the 6502 emulator. This contract specifies behavioral requirements that all implementations must satisfy.

## Trait Definition

```rust
/// Memory-mapped device trait.
///
/// Devices respond to read/write operations at offsets within their address space.
/// The memory mapper handles address translation and routing.
pub trait Device {
    /// Read a byte from the device at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset within device's address space (0 to size()-1)
    ///
    /// # Returns
    ///
    /// Byte value at the specified offset
    ///
    /// # Contract
    ///
    /// - MUST NOT panic
    /// - MUST return deterministic value for given offset and device state
    /// - MAY have side effects (e.g., clearing status flags)
    /// - Caller guarantees offset < size()
    fn read(&self, offset: u16) -> u8;

    /// Write a byte to the device at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - Offset within device's address space (0 to size()-1)
    /// * `value` - Byte value to write
    ///
    /// # Contract
    ///
    /// - MUST NOT panic
    /// - MAY ignore write (e.g., read-only registers)
    /// - MAY have side effects (e.g., setting flags, invoking callbacks)
    /// - Caller guarantees offset < size()
    fn write(&mut self, offset: u16, value: u8);

    /// Return the size of the device's address space in bytes.
    ///
    /// # Returns
    ///
    /// Number of addressable bytes (1 to 65536)
    ///
    /// # Contract
    ///
    /// - MUST return same value for lifetime of device
    /// - MUST return value > 0
    /// - MUST return value ≤ 65536
    fn size(&self) -> u16;
}
```

## Behavioral Requirements

### 1. No Panics

**Requirement**: All methods MUST handle invalid inputs gracefully without panicking.

**Rationale**: Matches 6502 hardware behavior (no bus errors). Enables WASM portability (panics abort WASM execution).

**Examples**:
- Invalid offset handled by caller (enforced by mapper)
- Read from write-only register returns last written value or default
- Write to read-only register is silently ignored

### 2. Determinism

**Requirement**: `read()` MUST return deterministic value given same offset and device state.

**Rationale**: Enables testing, debugging, and replay. Emulator behavior should be reproducible.

**Example**:
```rust
let device = MyDevice::new();
let val1 = device.read(0x00);
let val2 = device.read(0x00);
assert_eq!(val1, val2);  // Must pass if device state unchanged
```

**Exception**: Side effects may change state (e.g., reading clears flag), but behavior is still deterministic.

### 3. Immutable Reads

**Requirement**: `read()` takes `&self` (immutable reference).

**Rationale**: Allows shared read access. Maps to 6502 hardware (reads don't always change state).

**Note**: Devices with read side effects (e.g., clearing flags) use interior mutability (`Cell`, `RefCell` in single-threaded context).

### 4. Mutable Writes

**Requirement**: `write()` takes `&mut self` (mutable reference).

**Rationale**: Makes side effects explicit. Most writes modify device state (registers, buffers).

### 5. Constant Size

**Requirement**: `size()` MUST return same value for device lifetime.

**Rationale**: Mapper uses size for address range calculations. Changing size invalidates mappings.

**Enforcement**: Size should be determined at construction and never change.

## Implementation Examples

### Simple RAM Device

```rust
pub struct RamDevice {
    data: Vec<u8>,
}

impl RamDevice {
    pub fn new(size: u16) -> Self {
        Self {
            data: vec![0; size as usize],
        }
    }
}

impl Device for RamDevice {
    fn read(&self, offset: u16) -> u8 {
        self.data[offset as usize]
    }

    fn write(&mut self, offset: u16, value: u8) {
        self.data[offset as usize] = value;
    }

    fn size(&self) -> u16 {
        self.data.len() as u16
    }
}
```

**Satisfies contract**:
- ✅ No panics (offset guaranteed < size by caller)
- ✅ Deterministic (reads return written values)
- ✅ Immutable read (no state changes)
- ✅ Mutable write (modifies data)
- ✅ Constant size (vec length fixed at construction)

### Read-Only ROM Device

```rust
pub struct RomDevice {
    data: Vec<u8>,
}

impl RomDevice {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl Device for RomDevice {
    fn read(&self, offset: u16) -> u8 {
        self.data[offset as usize]
    }

    fn write(&mut self, offset: u16, value: u8) {
        // Silently ignore writes (read-only memory)
    }

    fn size(&self) -> u16 {
        self.data.len() as u16
    }
}
```

**Satisfies contract**:
- ✅ No panics
- ✅ Deterministic reads
- ✅ Writes ignored (valid behavior)
- ✅ Constant size

### Register-Based Device (UART)

```rust
pub struct Uart6551 {
    status_register: u8,
    data_register: u8,
    // ... other registers
}

impl Device for Uart6551 {
    fn read(&self, offset: u16) -> u8 {
        match offset {
            0 => self.read_data_register(),
            1 => self.status_register,
            2 => self.command_register,
            3 => self.control_register,
            _ => 0x00,  // Invalid offset
        }
    }

    fn write(&mut self, offset: u16, value: u8) {
        match offset {
            0 => self.write_data_register(value),
            1 => { /* Ignore writes to read-only status */ }
            2 => self.command_register = value,
            3 => self.control_register = value,
            _ => { /* Ignore invalid offset */ }
        }
    }

    fn size(&self) -> u16 {
        4  // Four registers
    }
}
```

**Satisfies contract**:
- ✅ No panics (invalid offsets handled)
- ✅ Deterministic (registers have defined values)
- ✅ Side effects allowed (data register read may clear flag)
- ✅ Constant size (always 4 bytes)

## Testing Contract Compliance

All Device implementations should include these test cases:

### 1. Size Consistency

```rust
#[test]
fn test_size_constant() {
    let device = MyDevice::new();
    let size1 = device.size();
    let size2 = device.size();
    assert_eq!(size1, size2);
}

#[test]
fn test_size_positive() {
    let device = MyDevice::new();
    assert!(device.size() > 0);
}
```

### 2. No Panics

```rust
#[test]
fn test_read_all_offsets() {
    let device = MyDevice::new();
    for offset in 0..device.size() {
        let _ = device.read(offset);  // Must not panic
    }
}

#[test]
fn test_write_all_offsets() {
    let mut device = MyDevice::new();
    for offset in 0..device.size() {
        device.write(offset, 0xFF);  // Must not panic
    }
}
```

### 3. Read Determinism (if stateless)

```rust
#[test]
fn test_read_determinism() {
    let device = MyDevice::new();
    let val1 = device.read(0);
    let val2 = device.read(0);
    assert_eq!(val1, val2);
}
```

### 4. Write-Read Consistency (if read/write both supported)

```rust
#[test]
fn test_write_read() {
    let mut device = MyDevice::new();
    device.write(0, 0x42);
    assert_eq!(device.read(0), 0x42);
}
```

## Integration with MappedMemory

The mapper uses the Device trait as follows:

```rust
impl MemoryBus for MappedMemory {
    fn read(&self, addr: u16) -> u8 {
        for mapping in &self.devices {
            if addr >= mapping.base_addr {
                let offset = addr - mapping.base_addr;
                if offset < mapping.device.size() {
                    return mapping.device.read(offset);
                }
            }
        }
        self.unmapped_value
    }

    fn write(&mut self, addr: u16, value: u8) {
        for mapping in &mut self.devices {
            if addr >= mapping.base_addr {
                let offset = addr - mapping.base_addr;
                if offset < mapping.device.size() {
                    mapping.device.write(offset, value);
                    return;
                }
            }
        }
    }
}
```

**Contract guarantees**:
- Caller ensures `offset < device.size()` before calling `read()`/`write()`
- Devices receive offsets starting from 0 (base address subtracted)
- Each address maps to at most one device (no overlaps)

## Common Pitfalls

### ❌ Returning Error Types

```rust
// WRONG: Device trait has no error handling
fn read(&self, offset: u16) -> Result<u8, Error> {
    if offset >= self.size() {
        return Err(Error::InvalidOffset);
    }
    Ok(self.data[offset as usize])
}
```

**Fix**: Trust mapper to validate offset. Handle invalid inputs gracefully:
```rust
fn read(&self, offset: u16) -> u8 {
    // Mapper guarantees offset < size(), but be defensive
    self.data.get(offset as usize).copied().unwrap_or(0xFF)
}
```

### ❌ Changing Size

```rust
// WRONG: Size changes after construction
fn expand(&mut self, new_size: u16) {
    self.data.resize(new_size as usize, 0);
}
```

**Fix**: Size is immutable. Create new device if size needs to change.

### ❌ Panicking on Invalid Input

```rust
// WRONG: Panics on write to read-only register
fn write(&mut self, offset: u16, value: u8) {
    panic!("Writes to ROM not supported!");
}
```

**Fix**: Silently ignore (matches hardware behavior):
```rust
fn write(&mut self, offset: u16, value: u8) {
    // Writes to ROM are no-ops
}
```

## Versioning

**Current Version**: 1.0.0

**Future Additions** (backward compatible):
- Optional `reset()` method for device initialization
- Optional `step()` method for devices with timing behavior
- Optional `interrupt_pending()` for IRQ/NMI sources

**Breaking Changes** (would require 2.0.0):
- Changing method signatures
- Adding required methods to trait
- Changing size() return type

## References

- MemoryBus trait: `src/memory.rs`
- Example implementations: `src/devices/ram.rs`, `src/devices/rom.rs`, `src/devices/uart.rs`
- Integration tests: `tests/memory_mapping_tests.rs`
