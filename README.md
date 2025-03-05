# simple-assembler

![lint-workflow](https://github.com/benoitlx/simple-assembler/actions/workflows/ci.yml/badge.svg)

Simple assembler for my custom cpu.

You should not be interested in this project unless you're curious about making your own assembler or you want to test out my [cpu](https://benoitlx.github.io/simple-cpu-wiki/Assembler/Assembler) (wip).

## Installation

It's only possible to install it from source for now.

Clone this repository and `cd` into it. Then run `cargo build --release` to build the binary.
You can place the binary `simple-assembler` located in `target/release` in your `PATH` in order to access it from everywhere.

## Usage

```
Usage: simple-assembler [OPTIONS] <FILE_PATH>

Arguments:
  <FILE_PATH>  assembly file path

Options:
  -c, --color                 whether to colorize the bit stream output
  -d, --debug                 whether to print debug messages
  -s, --sep <SEP>             separator between each words in the bit stream [default: ]
      --w-off                 whether to turn off warnings
  -W, --Warn                  whether to output the bit stream if warnings are encountered
  -o, --output <OUTPUT_PATH>  save output in designated file
  -h, --help                  Print help
```

## Example on a simple program 

```asm
DEFINE foo 0x7fff
DEFINE unused_var 42 ; warning here

main:
A = foo
D = *A
A = 5
D = D & A 

A = A ~ A ; error here

A = main
JMP
```

Running `simple-assembler prog.asm -c` on the code above gives us this output :

![Errors](/examples/output_with_error.png)

Commenting the error and the warning outputs the following :

![out](/examples/output.png)



## TODO

- [ ] Handle more error with miette (tokenization errors, ...)
- [ ] Integration test for parser
- [ ] Benchmark

## Acknowledgments

Here are the amazing crates I used to make this small project :
- [logos](https://github.com/maciejhirsz/logos) for tokenization
- [miette](https://github.com/zkat/miette) for error report
- [clap](https://github.com/clap-rs/clap) for parsing cli arguments
- [colored](https://github.com/colored-rs/colored) to colorize the output of the program
