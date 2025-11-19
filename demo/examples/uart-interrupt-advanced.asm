; UART Interrupt-Driven Advanced Example
; Demonstrates interrupt-driven UART with character counting
;
; Memory Map:
;   $A000 - UART Data Register (read/write)
;   $A001 - UART Status Register (read-only)
;   $A002 - UART Command Register (read/write)
;           Bit 1: IRQ_EN (Interrupt Enable)
;
; This program:
; 1. Receives characters via interrupt
; 2. Counts received characters in zero page
; 3. Echoes each character back
; 4. Shows ISR can maintain state

UART_DATA    = $A000     ; UART data register
UART_STATUS  = $A001     ; UART status register
UART_COMMAND = $A002     ; UART command register
IRQ_EN       = $02       ; Interrupt enable bit (bit 1)
CHAR_COUNT   = $00       ; Zero page counter

        ; Initialize character counter
        LDA #$00
        STA CHAR_COUNT

        ; Set up IRQ vector to point to our ISR
        LDA #<isr
        STA $FFFE
        LDA #>isr
        STA $FFFF

        ; Enable UART receive interrupts
        LDA #IRQ_EN
        STA UART_COMMAND

        ; Enable CPU interrupts
        CLI

        ; Main loop - display count periodically
main_loop:
        ; Main program can do other work here
        ; For demo, we just idle
        NOP
        JMP main_loop

        ; Interrupt Service Routine
        ; Called when UART receives a character
isr:
        ; Save accumulator (important in ISR!)
        PHA

        ; Read data from UART (clears interrupt)
        LDA UART_DATA

        ; Increment character counter
        INC CHAR_COUNT

        ; Echo the character back
        STA UART_DATA

        ; Check if we've received 10 characters
        LDA CHAR_COUNT
        CMP #$0A        ; 10 characters?
        BNE isr_done

        ; After 10 chars, send a newline and reset counter
        LDA #$0D        ; CR
        STA UART_DATA
        LDA #$0A        ; LF
        STA UART_DATA

        ; Reset counter
        LDA #$00
        STA CHAR_COUNT

isr_done:
        ; Restore accumulator
        PLA

        ; Return from interrupt
        RTI
