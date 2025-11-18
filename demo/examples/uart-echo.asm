; UART Echo
; Echoes characters typed in the terminal back to the terminal
;
; Memory Map:
;   $A000 - UART Data Register (read/write)
;   $A001 - UART Status Register (read-only)
;           Bit 3: RDRF (Receive Data Register Full)
;           Bit 4: TDRE (Transmit Data Register Empty)
;
; This program demonstrates a simple polling loop that:
; 1. Checks if a character is available (RDRF flag)
; 2. Reads the character from UART
; 3. Writes it back to UART (echoes it)
; 4. Repeats forever

UART_DATA   = $A000     ; UART data register
UART_STATUS = $A001     ; UART status register
RDRF        = $08       ; Receive Data Register Full flag (bit 3)

        ; Main echo loop
echo_loop:
        LDA UART_STATUS ; Read UART status register
        AND #RDRF       ; Check RDRF flag (bit 3)
        BEQ echo_loop   ; If no data, keep polling

        ; Character is available - read it
        LDA UART_DATA   ; Read character from UART

        ; Echo it back
        STA UART_DATA   ; Write character back to UART

        ; Loop forever
        JMP echo_loop
