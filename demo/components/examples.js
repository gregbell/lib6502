/**
 * Example Selector Component
 * Load pre-written example programs
 */

export class ExampleSelector {
    constructor(editor) {
        this.editor = editor;
        this.examples = this.getExamples();
        this.render();
        this.setupEventListeners();
    }

    getExamples() {
        return [
            {
                id: 'counter',
                name: 'Counter',
                description: 'Simple counter from 0 to 255',
                code: `; Simple Counter
; Counts from 0 to 255 in the accumulator

        LDA #$00        ; Start at 0
loop:
        CLC             ; Clear carry
        ADC #$01        ; Add 1
        BNE loop        ; Loop if not zero
        BRK             ; Done`
            },
            {
                id: 'fibonacci',
                name: 'Fibonacci',
                description: 'Calculate Fibonacci sequence',
                code: `; Fibonacci Sequence
; Calculates Fibonacci numbers

        LDA #$00        ; F(0) = 0
        STA $00
        LDA #$01        ; F(1) = 1
        STA $01

loop:
        LDA $00         ; Load F(n)
        CLC
        ADC $01         ; Add F(n+1)
        STA $02         ; Store F(n+2)

        LDA $01         ; Shift: F(n) = F(n+1)
        STA $00
        LDA $02         ; F(n+1) = F(n+2)
        STA $01

        BCC loop        ; Continue if no overflow
        BRK             ; Done`
            },
            {
                id: 'stack',
                name: 'Stack Demo',
                description: 'Demonstrate stack operations',
                code: `; Stack Operations Demo
; Shows PHA/PLA stack usage

        LDA #$42        ; Load value
        PHA             ; Push A to stack

        LDA #$00        ; Clear A

        PLA             ; Pull from stack back to A

        TAX             ; Transfer to X
        TAY             ; Transfer to Y

        BRK             ; Done`
            },
            {
                id: 'uart-echo',
                name: 'UART Echo (IRQ)',
                description: 'Interrupt-driven echo example',
                code: `; UART Echo (Interrupt-Driven)
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
        RTI                 ; Restores PC and status, continues main loop`
            },
            {
                id: 'uart-hello',
                name: 'UART Hello World',
                description: 'Output "Hello, 6502!" to terminal',
                code: `; UART Hello World
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
        .byte $00       ; Null terminator`
            },
            {
                id: 'uart-interrupt-advanced',
                name: 'UART IRQ Advanced',
                description: 'Interrupt-driven with character counting',
                code: `; UART Interrupt-Driven Advanced Example
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
        RTI`
            }
        ];
    }

    render() {
        const container = document.querySelector('.editor-panel');
        if (!container) return;

        const selectorHtml = `
            <div class="example-selector">
                <label for="example-select">Examples:</label>
                <select id="example-select">
                    <option value="">-- Select an example --</option>
                    ${this.examples.map(ex =>
                        `<option value="${ex.id}">${ex.name} - ${ex.description}</option>`
                    ).join('')}
                </select>
            </div>
        `;

        // Insert before editor container
        const editorContainer = document.getElementById('editor-container');
        editorContainer.insertAdjacentHTML('beforebegin', selectorHtml);
    }

    setupEventListeners() {
        const select = document.getElementById('example-select');
        if (!select) return;

        select.addEventListener('change', (e) => {
            const exampleId = e.target.value;
            if (!exampleId) return;

            const example = this.examples.find(ex => ex.id === exampleId);
            if (example) {
                this.editor.setValue(example.code);
                document.dispatchEvent(new CustomEvent('example-loaded', { detail: example }));
                console.log(`✓ Loaded example: ${example.name}`);
            }
        });
    }
}
