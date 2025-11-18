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
                name: 'UART Echo',
                description: 'Echo characters from terminal',
                code: `; UART Echo
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
        JMP echo_loop`
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
;           Bit 4: TDRE (Transmit Data Register Empty)
;
; This program demonstrates string output by:
; 1. Loading a pointer to the message string
; 2. For each character:
;    a. Wait for TDRE (transmit ready)
;    b. Write character to UART
; 3. Stop when null terminator ($00) is reached

UART_DATA   = $A000     ; UART data register
UART_STATUS = $A001     ; UART status register
TDRE        = $10       ; Transmit Data Register Empty flag (bit 4)

        ; Initialize pointer to message
        LDX #$00        ; X register will index into message

print_loop:
        ; Load next character
        LDA message,X   ; Load character from message
        BEQ done        ; If zero (null terminator), we're done

        ; Wait for transmitter ready
wait_tx:
        PHA             ; Save character on stack
        LDA UART_STATUS ; Read UART status
        AND #TDRE       ; Check TDRE flag (bit 4)
        BEQ wait_tx     ; If not ready, keep waiting
        PLA             ; Restore character

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
                id: 'uart-polling',
                name: 'UART Polling',
                description: 'Demonstrate UART status polling',
                code: `; UART Polling Demo
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
        BRK             ; Stop execution`
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
