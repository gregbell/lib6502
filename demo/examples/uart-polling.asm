; UART Polling Demo
; Demonstrates UART status register polling techniques
;
; Memory Map:
;   $A000 - UART Data Register (read/write)
;   $A001 - UART Status Register (read-only)
;           Bit 3: RDRF (Receive Data Register Full)
;           Bit 4: TDRE (Transmit Data Register Empty)
;
; This program demonstrates:
; 1. Polling RDRF to detect incoming data
; 2. Polling TDRE to ensure transmitter is ready
; 3. Safe read/write patterns with status checking
;
; The program reads one character, then outputs it 3 times
; to demonstrate controlled UART operations.

UART_DATA   = $A000     ; UART data register
UART_STATUS = $A001     ; UART status register
RDRF        = $08       ; Receive Data Register Full flag (bit 3)
TDRE        = $10       ; Transmit Data Register Empty flag (bit 4)

        ; Wait for incoming character
wait_rx:
        LDA UART_STATUS ; Read UART status register
        AND #RDRF       ; Check RDRF flag (bit 3)
        BEQ wait_rx     ; Loop until character available

        ; Read the character
        LDA UART_DATA   ; Read character (also clears RDRF)
        PHA             ; Save character on stack

        ; Output the character 3 times with polling
        LDX #$03        ; Counter for 3 repetitions

output_loop:
        ; Wait for transmitter ready
poll_tx:
        LDA UART_STATUS ; Read UART status register
        AND #TDRE       ; Check TDRE flag (bit 4)
        BEQ poll_tx     ; Loop until transmitter ready

        ; Transmitter is ready - send character
        PLA             ; Get character from stack
        STA UART_DATA   ; Write to UART data register
        PHA             ; Put character back on stack

        ; Decrement counter and continue
        DEX             ; Decrease counter
        BNE output_loop ; Continue if not zero

        ; Clean up and finish
        PLA             ; Remove character from stack
        BRK             ; Stop execution
