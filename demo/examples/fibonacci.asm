; Fibonacci Sequence
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
        BRK             ; Done
