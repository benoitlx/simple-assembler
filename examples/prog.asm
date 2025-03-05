DEFINE foo 0x7fff
; DEFINE unused_var 42 ; warning here

main:
A = foo
D = *A
A = 5
D = D & A 

; A = A ~ A ; error here

A = main
JMP
