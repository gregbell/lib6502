# BUG: ADC and SBC Don't Implement Decimal Mode (BCD Arithmetic)

**Issue Type:** Bug - Missing Feature
**Severity:** High - Blocks Klaus Functional Test Completion
**Discovered By:** Klaus Dormann's 6502 Functional Test Suite

## Problem Summary

The ADC (Add with Carry) and SBC (Subtract with Carry) instructions do not implement Binary Coded Decimal (BCD) arithmetic when the Decimal flag (D) is set. This causes the Klaus functional test to fail at 99.6% completion.

## Evidence

### Klaus Test Failure Details

```
=== Klaus 6502 Functional Test ===
Entry point: $0400
Success address: $3469
Test completed!
Final PC: $3477 (expected $3469)

CPU State at Failure:
  A: $33
  Flags: NVD---B (Decimal mode SET ✓)
  Cycles: 84,024,460

Failure Location: PC $3477
  $3472: ADC $0E    ; Decimal mode addition
  $3475: CMP $0F    ; Compare with expected result
  $3477: BNE *      ; TRAPPED HERE - result didn't match expected
```

### Test Context

From `6502_functional_test.lst`:
```assembly
346f : chkdad
     ; decimal ADC / SBC zp
346f : 08         php             ; save carry for subtract
3470 : a50d       lda ad1
3472 : 650e       adc ad2         ; perform add
3474 : 08         php
3475 : c50f       cmp adrl        ; check result
3477 : d0fe       bne *           ; failed not equal (non zero)
```

The test sets the D flag, performs decimal ADC, and expects BCD-adjusted results. Our implementation performs binary addition instead.

## What's Working

✅ All binary mode arithmetic (ADC, SBC work correctly with D flag clear)
✅ 84+ million instruction cycles executed successfully
✅ All other 6502 functionality (99.6% of test passes)
✅ 1,453 existing tests pass

## What's Broken

❌ ADC with D flag set doesn't adjust result to BCD format
❌ SBC with D flag set doesn't adjust result to BCD format

## Technical Details

### Binary Coded Decimal (BCD) Mode

When the Decimal flag (D) is set:
- Each byte represents two decimal digits (00-99)
- Each nibble (4 bits) holds one digit (0-9)
- Arithmetic operations must keep nibbles in range 0-9
- Carries/borrows occur at decimal boundaries (10, not 16)

### Examples

**ADC in Decimal Mode:**
```
Example 1: $09 + $01
  Binary mode:   $09 + $01 = $0A (wrong in decimal)
  Decimal mode:  $09 + $01 = $10 (9 + 1 = 10, adjust to BCD)

Example 2: $58 + $47
  Binary mode:   $58 + $47 = $9F (wrong)
  Decimal mode:  $58 + $47 = $05 (with carry) (58 + 47 = 105)
```

**SBC in Decimal Mode:**
```
Example: $50 - $25 (with carry set)
  Binary mode:   $50 - $25 = $2B (wrong)
  Decimal mode:  $50 - $25 = $25 (50 - 25 = 25)
```

### BCD Adjustment Algorithm

**ADC (pseudocode):**
```rust
if cpu.flag_d {
    // Perform binary addition first
    let result = a + value + carry_in;

    // Adjust low nibble
    let mut adjusted = result;
    if (result & 0x0F) > 0x09 {
        adjusted += 0x06;
    }

    // Adjust high nibble
    if (adjusted & 0xF0) > 0x90 {
        adjusted += 0x60;
    }

    // Set carry if result > $99
    cpu.flag_c = result > 0x99;
    cpu.a = adjusted as u8;

    // Note: N, V, Z flags behave differently in decimal mode
} else {
    // Current binary mode implementation (correct)
}
```

**SBC (similar adjustment needed):**
- Perform binary subtraction
- Adjust nibbles to stay in 0-9 range
- Borrow at decimal boundaries

### NMOS 6502 Decimal Mode Quirks

Important implementation notes:
1. **N and V flags**: On NMOS 6502, these are **undefined** in decimal mode
   - Some docs say they reflect binary operation result
   - Some say they're unpredictable
   - Klaus test may or may not check these

2. **Z flag**: Set normally (if result is $00)

3. **C flag**: Reflects decimal carry (not binary)

4. **Timing**: Decimal mode has same cycle counts as binary mode

## Files to Modify

### Primary Changes

1. **src/instructions/alu.rs** - `execute_adc()` function (~line 40)
   - Add check for `cpu.flag_d`
   - Implement BCD adjustment when decimal mode is active
   - Keep existing binary mode logic

2. **src/instructions/alu.rs** - `execute_sbc()` function
   - Add check for `cpu.flag_d`
   - Implement BCD subtraction adjustment
   - Keep existing binary mode logic

### Testing

3. **tests/adc_test.rs** - Add decimal mode tests
   - Test various BCD additions
   - Test carry propagation in decimal mode
   - Test edge cases ($99 + $01, etc.)

4. **tests/sbc_test.rs** - Add decimal mode tests
   - Test various BCD subtractions
   - Test borrow propagation in decimal mode
   - Test edge cases

## Acceptance Criteria

- [ ] ADC correctly performs BCD arithmetic when `cpu.flag_d == true`
- [ ] SBC correctly performs BCD arithmetic when `cpu.flag_d == true`
- [ ] Binary mode arithmetic still works (flag_d == false)
- [ ] All existing tests continue to pass (1,453 tests)
- [ ] Klaus functional test passes completely (PC reaches $3469)
- [ ] New decimal mode tests added and passing

## References

- **Klaus Test Suite**: https://github.com/Klaus2m5/6502_65C02_functional_tests
- **6502 Decimal Mode**: http://www.6502.org/tutorials/decimal_mode.html
- **BCD Arithmetic**: http://www.righto.com/2012/08/reverse-engineering-bcd-on-6502.html
- **Test Results**: See `docs/KLAUS_FUNCTIONAL_TEST.md`

## Reproduction Steps

```bash
# Run Klaus functional test (currently fails at decimal mode)
cargo test --test functional_klaus klaus_6502_functional_test -- --ignored --nocapture

# Expected: Test fails at PC $3477
# After fix: Test should pass at PC $3469
```

## Impact

This is the **only remaining gap** in 6502 instruction implementation:
- 99.6% of functionality works
- 84 million cycles of complex code executes correctly
- Only decimal mode arithmetic missing

Fixing this will achieve 100% Klaus test compliance and complete NMOS 6502 emulation.

## Priority

**High** - This is the last barrier to complete 6502 emulation and full test suite passage.

## Estimated Effort

- **Medium** (4-8 hours)
- BCD algorithm is well-documented
- Main complexity: handling edge cases and flag behavior
- Testing will take significant time due to many combinations

---

**Discovered**: 2024 (via Klaus functional test integration)
**Affects**: ADC (8 opcodes), SBC (8 opcodes) - 16 total opcodes when D flag set
**Workaround**: None - decimal mode is required for test completion
