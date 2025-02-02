// testing decimal numbers
DEFINE titi 0
DEFINE tito 1
DEFINE toto 32767
// DEFINE tata 32768 // error here
// DEFINE big_number 99999999999999 // error here

// testing hexadecimal numbers
DEFINE a 0xff
DEFINE b 0x0
DEFINE c 0x7fff
// DEFINE d 0x8001 // error here

// testing binary numbers
DEFINE d 0b0
DEFINE e 0b11001
DEFINE f 0b111111111111111
// DEFINE g 0b1111111111111111 // error here

// error below
// DEFINE foo 0x0
// DEFINE foo 0x0

// DEFINE test // error here
// DEFINE many 0x0 argument // error here

// DEFINE test test // error here
// DEFINE t*est 0x0 // error here
// DEFINE t/est 0x0 // error here

A:
