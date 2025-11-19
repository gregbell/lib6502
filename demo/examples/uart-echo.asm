; UART Echo (Interrupt-Driven)
; Echoes characters typed in the terminal back to the terminal using IRQ
;
; Memory Map:
;   $A000 - UART Data Register (read/write)
;   $A001 - UART Status Register (read-only)
;   $A002 - UART Command Register (read/write)
;           Bit 1: IRQ_EN (Interrupt Enable)
;           Bit 3: ECHO (Echo mode)
;
; This program demonstrates interrupt-driven serial I/O:
; 1. Configure UART to trigger interrupts on receive
; 2. Enable CPU interrupts (CLI)
; 3. When data arrives, CPU jumps to ISR automatically
; 4. ISR reads data and echoes it back
; 5. RTI returns to main loop

UART_DATA    = $A000     ; UART data register
UART_STATUS  = $A001     ; UART status register
UART_COMMAND = $A002     ; UART command register
IRQ_EN       = $02       ; Interrupt enable bit (bit 1)

        ; Set up IRQ vector to point to our ISR
        LDA #<isr           ; Low byte of ISR address
        STA $FFFE
        LDA #>isr           ; High byte of ISR address
        STA $FFFF

        ; Enable UART receive interrupts
        LDA #IRQ_EN         ; Set bit 1 (interrupt enable)
        STA UART_COMMAND

        ; Enable CPU interrupts
        CLI                 ; Clear interrupt disable flag

        ; Main loop - CPU is now free to do other work
        ; Interrupts will be serviced automatically
idle_loop:
        NOP                 ; CPU idles here between interrupts
        JMP idle_loop

        ; Interrupt Service Routine
        ; Called automatically when UART receives data
isr:
        ; Read data from UART (clears interrupt automatically)
        LDA UART_DATA

        ; Echo character back to terminal
        STA UART_DATA

        ; Return from interrupt
        RTI                 ; Restores PC and status, continues main loop
