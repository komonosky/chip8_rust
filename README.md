# Introduction
This is a CHIP-8 emulator/interpreter written in Rust. CHIP-8 is an interpreted programming language developed in the mid-1970s. It was initially used on the COSMAC VIP and Telmac 1800 8-bit microcomputers. CHIP-8 was made to allow video games to be more easily programmed for these computers. The programs were run on a CHIP-8 virtual machine.

CHIP-8 is an excellent introduction to emulator development, as it has a relatively small number of instructions compared with more recent systems such as the Gameboy or NES. 

# Requirements
You will need to have SDL2 installed. Get it here: https://www.libsdl.org/download-2.0.php. (Figuring out how to install SDL2 and get it set up was probably the most challenging part  of this project).

# How to run
Clone the repository, then run: 
```
cargo run path/to/game
```
Some games are included in the chip8_frontend folder.

The early computers that used CHIP-8 had a 16-key hexadecimal keypad with the following layout:

1	2	3	C
4	5	6	D
7	8	9	E
A	0	B	F

I have mapped this to a modern QWERTY keyboard the following way:

1 2 3 4
Q W E R
A S D F
Z X C V
