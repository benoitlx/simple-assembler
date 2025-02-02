DEFINE io_adr 0x7fff
DEFINE ob_detection_mask 0x100
DEFINE movement_mask 0x600
DEFINE move_mask 0x4
DEFINE left 0x8


wait:
A = movement_mask
D = A
A = io_adr
D = D & *A
A = wait
D ; JNE

check:
A = ob_detection_mask
D = A
A = io_adr
D = D & *A
A = move
D ; JEQ

turn:
A = left
D = A

send:
A = io_adr
*A = D
A = wait
JMP

move:
A = move_mask
D = A
A = send
JMP
D ; this is a comment 
D