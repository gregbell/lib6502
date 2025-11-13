## **The Instruction Set**

The 6502 has a relatively basic set of instructions, many having similar
functions (e.g. memory access, arithmetic, etc.). The following sections
list the complete set of 56 instructions in functional groups.

### Load/Store Operations

These instructions transfer a single byte between memory and one of the
registers. Load operations set the negative
([N](http://www.6502.org/users/obelisk/6502/registers.html#N)) and zero
([Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)) flags
depending on the value of transferred. Store operations do not affect
the flag settings.

  ------------------------------------------------------------------ ------------------- ---------------------------------------------------------------------------------------------------------------------------
  [LDA](http://www.6502.org/users/obelisk/6502/reference.html#LDA)   Load Accumulator    [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [LDX](http://www.6502.org/users/obelisk/6502/reference.html#LDX)   Load X Register     [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [LDY](http://www.6502.org/users/obelisk/6502/reference.html#LDY)   Load Y Register     [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [STA](http://www.6502.org/users/obelisk/6502/reference.html#STA)   Store Accumulator    
  [STX](http://www.6502.org/users/obelisk/6502/reference.html#STX)   Store X Register     
  [STY](http://www.6502.org/users/obelisk/6502/reference.html#STY)   Store Y Register     
  ------------------------------------------------------------------ ------------------- ---------------------------------------------------------------------------------------------------------------------------

### Register Transfers

The contents of the X and Y registers can be moved to or from the
accumulator, setting the negative
([N](http://www.6502.org/users/obelisk/6502/registers.html#N)) and zero
([Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)) flags as
appropriate.

  ------------------------------------------------------------------ --------------------------- ---------------------------------------------------------------------------------------------------------------------------
  [TAX](http://www.6502.org/users/obelisk/6502/reference.html#TAX)   Transfer accumulator to X   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [TAY](http://www.6502.org/users/obelisk/6502/reference.html#TAY)   Transfer accumulator to Y   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [TXA](http://www.6502.org/users/obelisk/6502/reference.html#TXA)   Transfer X to accumulator   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [TYA](http://www.6502.org/users/obelisk/6502/reference.html#TYA)   Transfer Y to accumulator   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  ------------------------------------------------------------------ --------------------------- ---------------------------------------------------------------------------------------------------------------------------

### Stack Operations

The 6502 microprocessor supports a 256 byte stack fixed between memory
locations \$0100 and \$01FF. A special 8-bit register, S, is used to
keep track of the next free byte of stack space. Pushing a byte on to
the stack causes the value to be stored at the current free location
(e.g. \$0100,S) and then the stack pointer is post decremented. Pull
operations reverse this procedure.

The stack register can only be accessed by transferring its value to or
from the X register. Its value is automatically modified by push/pull
instructions, subroutine calls and returns, interrupts and returns from
interrupts.

  ------------------------------------------------------------------ ---------------------------------- ---------------------------------------------------------------------------------------------------------------------------
  [TSX](http://www.6502.org/users/obelisk/6502/reference.html#TSX)   Transfer stack pointer to X        [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [TXS](http://www.6502.org/users/obelisk/6502/reference.html#TXS)   Transfer X to stack pointer         
  [PHA](http://www.6502.org/users/obelisk/6502/reference.html#PHA)   Push accumulator on stack           
  [PHP](http://www.6502.org/users/obelisk/6502/reference.html#PHP)   Push processor status on stack      
  [PLA](http://www.6502.org/users/obelisk/6502/reference.html#PLA)   Pull accumulator from stack        [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [PLP](http://www.6502.org/users/obelisk/6502/reference.html#PLP)   Pull processor status from stack   All
  ------------------------------------------------------------------ ---------------------------------- ---------------------------------------------------------------------------------------------------------------------------

### Logical

The following instructions perform logical operations on the contents of
the accumulator and another value held in memory. The BIT instruction
performs a logical AND to test the presence of bits in the memory value
to set the flags but does not keep the result.

  ------------------------------------------------------------------ ---------------------- ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
  [AND](http://www.6502.org/users/obelisk/6502/reference.html#AND)   Logical AND            [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [EOR](http://www.6502.org/users/obelisk/6502/reference.html#EOR)   Exclusive OR           [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [ORA](http://www.6502.org/users/obelisk/6502/reference.html#ORA)   Logical Inclusive OR   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [BIT](http://www.6502.org/users/obelisk/6502/reference.html#BIT)   Bit Test               [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[V](http://www.6502.org/users/obelisk/6502/registers.html#V),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  ------------------------------------------------------------------ ---------------------- ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

### Arithmetic

The arithmetic operations perform addition and subtraction on the
contents of the accumulator. The compare operations allow the comparison
of the accumulator and X or Y with memory values.

  ------------------------------------------------------------------ --------------------- -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
  [ADC](http://www.6502.org/users/obelisk/6502/reference.html#ADC)   Add with Carry        [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[V](http://www.6502.org/users/obelisk/6502/registers.html#V),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [SBC](http://www.6502.org/users/obelisk/6502/reference.html#SBC)   Subtract with Carry   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[V](http://www.6502.org/users/obelisk/6502/registers.html#V),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [CMP](http://www.6502.org/users/obelisk/6502/reference.html#CMP)   Compare accumulator   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [CPX](http://www.6502.org/users/obelisk/6502/reference.html#CPX)   Compare X register    [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [CPY](http://www.6502.org/users/obelisk/6502/reference.html#CPY)   Compare Y register    [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  ------------------------------------------------------------------ --------------------- -----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

### Increments & Decrements

Increment or decrement a memory location or one of the X or Y registers
by one setting the negative
([N](http://www.6502.org/users/obelisk/6502/registers.html#N)) and zero
([Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)) flags as
appropriate,

  ------------------------------------------------------------------ ----------------------------- ---------------------------------------------------------------------------------------------------------------------------
  [INC](http://www.6502.org/users/obelisk/6502/reference.html#INC)   Increment a memory location   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [INX](http://www.6502.org/users/obelisk/6502/reference.html#INX)   Increment the X register      [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [INY](http://www.6502.org/users/obelisk/6502/reference.html#INY)   Increment the Y register      [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [DEC](http://www.6502.org/users/obelisk/6502/reference.html#DEC)   Decrement a memory location   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  [DEX](http://www.6502.org/users/obelisk/6502/reference.html#DEX)   Decrement the X register      [N](#N),[Z](#Z)
  [DEY](http://www.6502.org/users/obelisk/6502/reference.html#DEY)   Decrement the Y register      [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z)
  ------------------------------------------------------------------ ----------------------------- ---------------------------------------------------------------------------------------------------------------------------

### Shifts

Shift instructions cause the bits within either a memory location or the
accumulator to be shifted by one bit position. The rotate instructions
use the contents if the carry flag
([C](http://www.6502.org/users/obelisk/6502/registers.html#C)) to fill
the vacant position generated by the shift and to catch the overflowing
bit. The arithmetic and logical shifts shift in an appropriate 0 or 1
bit as appropriate but catch the overflow bit in the carry flag
([C](http://www.6502.org/users/obelisk/6502/registers.html#C)).

  ------------------------------------------------------------------ ----------------------- ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
  [ASL](http://www.6502.org/users/obelisk/6502/reference.html#ASL)   Arithmetic Shift Left   [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [LSR](http://www.6502.org/users/obelisk/6502/reference.html#LSR)   Logical Shift Right     [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [ROL](http://www.6502.org/users/obelisk/6502/reference.html#ROL)   Rotate Left             [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [ROR](http://www.6502.org/users/obelisk/6502/reference.html#ROR)   Rotate Right            [N](http://www.6502.org/users/obelisk/6502/registers.html#N),[Z](http://www.6502.org/users/obelisk/6502/registers.html#Z),[C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  ------------------------------------------------------------------ ----------------------- ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

### Jumps & Calls

The following instructions modify the program counter causing a break to
normal sequential execution. The
[JSR](http://www.6502.org/users/obelisk/6502/reference.html#JSR)
instruction pushes the old
[PC](http://www.6502.org/users/obelisk/6502/registers.html#PC) onto the
stack before changing it to the new location allowing a subsequent
[RTS](http://www.6502.org/users/obelisk/6502/reference.html#RTS) to
return execution to the instruction after the call.

  ------------------------------------------------------------------ -------------------------- ---
  [JMP](http://www.6502.org/users/obelisk/6502/reference.html#JMP)   Jump to another location    
  [JSR](http://www.6502.org/users/obelisk/6502/reference.html#JSR)   Jump to a subroutine        
  [RTS](http://www.6502.org/users/obelisk/6502/reference.html#RTS)   Return from subroutine      
  ------------------------------------------------------------------ -------------------------- ---

### Branches

Branch instructions break the normal sequential flow of execution by
changing the program counter if a specified condition is met. All the
conditions are based on examining a single bit within the processor
status.

  ------------------------------------------------------------------ ------------------------------- ---
  [BCC](http://www.6502.org/users/obelisk/6502/reference.html#BCC)   Branch if carry flag clear       
  [BCS](http://www.6502.org/users/obelisk/6502/reference.html#BCS)   Branch if carry flag set         
  [BEQ](http://www.6502.org/users/obelisk/6502/reference.html#BEQ)   Branch if zero flag set          
  [BMI](http://www.6502.org/users/obelisk/6502/reference.html#BMI)   Branch if negative flag set      
  [BNE](http://www.6502.org/users/obelisk/6502/reference.html#BNE)   Branch if zero flag clear        
  [BPL](http://www.6502.org/users/obelisk/6502/reference.html#BPL)   Branch if negative flag clear    
  [BVC](http://www.6502.org/users/obelisk/6502/reference.html#BVC)   Branch if overflow flag clear    
  [BVS](http://www.6502.org/users/obelisk/6502/reference.html#BVS)   Branch if overflow flag set      
  ------------------------------------------------------------------ ------------------------------- ---

Branch instructions use relative address to identify the target
instruction if they are executed. As relative addresses are stored using
a signed 8 bit byte the target instruction must be within 126 bytes
before the branch or 128 bytes after the branch.

### Status Flag Changes

The following instructions change the values of specific status flags.

  ------------------------------------------------------------------ ------------------------------ --------------------------------------------------------------
  [CLC](http://www.6502.org/users/obelisk/6502/reference.html#CLC)   Clear carry flag               [C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [CLD](http://www.6502.org/users/obelisk/6502/reference.html#CLD)   Clear decimal mode flag        [D](http://www.6502.org/users/obelisk/6502/registers.html#D)
  [CLI](http://www.6502.org/users/obelisk/6502/reference.html#CLI)   Clear interrupt disable flag   [I](http://www.6502.org/users/obelisk/6502/registers.html#I)
  [CLV](http://www.6502.org/users/obelisk/6502/reference.html#CLV)   Clear overflow flag            [V](http://www.6502.org/users/obelisk/6502/registers.html#V)
  [SEC](http://www.6502.org/users/obelisk/6502/reference.html#SEC)   Set carry flag                 [C](http://www.6502.org/users/obelisk/6502/registers.html#C)
  [SED](http://www.6502.org/users/obelisk/6502/reference.html#SED)   Set decimal mode flag          [D](http://www.6502.org/users/obelisk/6502/registers.html#D)
  [SEI](http://www.6502.org/users/obelisk/6502/reference.html#SEI)   Set interrupt disable flag     [I](http://www.6502.org/users/obelisk/6502/registers.html#I)
  ------------------------------------------------------------------ ------------------------------ --------------------------------------------------------------

### System Functions

The remaining instructions perform useful but rarely used functions.

  ------------------------------------------------------------------ ----------------------- --------------------------------------------------------------
  [BRK](http://www.6502.org/users/obelisk/6502/reference.html#BRK)   Force an interrupt      [B](http://www.6502.org/users/obelisk/6502/registers.html#B)
  [NOP](http://www.6502.org/users/obelisk/6502/reference.html#NOP)   No Operation             
  [RTI](http://www.6502.org/users/obelisk/6502/reference.html#RTI)   Return from Interrupt   All
  ------------------------------------------------------------------ ----------------------- --------------------------------------------------------------

  --------------------------------------------------------------------- ------------------------------------------------------------------------ --------------------------------------------------------------- ---------------------------------------------------------------------
   [\<\< Back](http://www.6502.org/users/obelisk/6502/registers.html)   [Home](http://www.6502.org/users/obelisk/index.html){target="_parent"}   [Contents](http://www.6502.org/users/obelisk/6502/index.html)   [Next \>\>](http://www.6502.org/users/obelisk/6502/addressing.html)
  --------------------------------------------------------------------- ------------------------------------------------------------------------ --------------------------------------------------------------- ---------------------------------------------------------------------

------------------------------------------------------------------------

This page was last updated on 2nd January 2002
