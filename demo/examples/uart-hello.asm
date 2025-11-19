; UART Hello World
; Outputs "Hello, 6502!" followed by a newline to the terminal
;
; Memory Map:
;   $A000 - UART Data Register (read/write)
;   $A001 - UART Status Register (read-only)
;           Bit 4: TDRE (Transmit Data Register Empty - always 1)
;
; This program demonstrates simple string output.
; No interrupts needed for transmit since TDRE is always ready.

UART_DATA   = $A000     ; UART data register
UART_STATUS = $A001     ; UART status register
TDRE        = $10       ; Transmit Data Register Empty flag (bit 4)

        ; Initialize pointer to message
        LDX #$00        ; X register will index into message

print_loop:
        ; Load next character
        LDA message,X   ; Load character from message
        BEQ done        ; If zero (null terminator), we're done

        ; TDRE is always set in our emulator, but we show
        ; the check pattern for completeness
        BIT UART_STATUS ; Check status (V flag set if bit 6 is set)

        ; Send character
        STA UART_DATA   ; Write character to UART

        ; Move to next character
        INX             ; Increment index
        JMP print_loop  ; Continue with next character

done:
        BRK             ; Stop execution

        ; Message string (null-terminated)
message:
        .byte "Hello, 6502!"
        .byte $0D, $0A  ; CR, LF (newline)
        .byte $00       ; Null terminator
