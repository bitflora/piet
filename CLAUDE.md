# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust interpreter for Pietxt, a text-based variant of the [Piet esoteric programming language](https://www.dangermouse.net/esoteric/piet.html). The original Piet uses images with colored pixels; this project uses plain text files for easier prototyping before converting to the visual format.

## Commands

```bash
cargo build              # Build
cargo run                # Run default program (program.txt)
cargo run -- myfile.txt  # Run a specific program file
cargo run --bin pietc myfile.txt # compile a program to a gif
cargo test               # Run all tests
cargo test -- --nocapture  # Run tests with stdout visible
```

## Architecture

Everything lives in `src/main.rs`. The interpreter is intentionally simple with no external dependencies.

## Pietxt

The following describes a stack-based language, based off Piet. Each line of the program contains a single instruction, optionally followed by a space then a number, which would the value passed to the command. '#' begins a comment; everything from there to the end of the line is ignored.

### Commands
- push x [variable_name]: Pushes x on to the stack. Accepts an optional variable name to more easily track this value across the stack.
- pop: Pops the top value off the stack and discards it.
- add [variable_name]: Pops the top two values off the stack, adds them, and pushes the result back on the stack. Accepts an optional variable name to more easily track the result across the stack.
- subtract [variable_name]: Pops the top two values off the stack, calculates the second top value minus the top value, and pushes the result back on the stack. Accepts an optional variable name to more easily track the result across the stack.
- multiply [variable_name]: Pops the top two values off the stack, multiplies them, and pushes the result back on the stack. Accepts an optional variable name to more easily track the result across the stack.
- divide [variable_name]: Pops the top two values off the stack, calculates the integer division of the second top value by the top value, and pushes the result back on the stack. If a divide by zero occurs, it is handled as an implementation-dependent error, though simply ignoring the command is recommended. Accepts an optional variable name to more easily track the result across the stack.
- mod [variable_name]: Pops the top two values off the stack, calculates the second top value modulo the top value, and pushes the result back on the stack. The result has the same sign as the divisor (the top value). If the top value is zero, this is a divide by zero error, which is handled as an implementation-dependent error, though simply ignoring the command is recommended. (See note below.) Accepts an optional variable name to more easily track the result across the stack.
- not [variable_name]: Replaces the top value of the stack with 0 if it is non-zero, and 1 if it is zero. Accepts an optional variable name to more easily track the result across the stack.
- greater [variable_name]: Pops the top two values off the stack, and pushes 1 on to the stack if the second top value is greater than the top value, and pushes 0 if it is not greater.
- duplicate: Pushes a copy of the top value on the stack on to the stack. Accepts an optional variable name to more easily track the result across the stack.
- roll: Pops the top two values off the stack and "rolls" the remaining stack entries to a depth equal to the second value popped, by a number of rolls equal to the first value popped. A single roll to depth n is defined as burying the top value on the stack n deep and bringing all values above it up by 1 place. A negative number of rolls rolls in the opposite direction. A negative depth is an error and the command is ignored. If a roll is greater than an implementation-dependent maximum stack depth, it is handled as an implementation-dependent error, though simply ignoring the command is recommended. To get an entry `X` deep in the stack (ignoring the parameters to `roll` itself), push `X` and then push `X-1`, then roll.
- out_number: Pops the top value off the stack and prints it to STDOUT as a number.
- out_char: Pops the top value off the stack and prints it to STDOUT as the equivalent ascii character.
- branch x: Pops the top value off the stack. If that value is non-zero, it jumps to the line number indicated by x. Line numbers are zero-indexed. Of course, in real Piet you would need to implement this in the structure of your program by manipulating the DP/CC, but alas, text is not sophisticated enough to capture this.
- debug_stack: Prints the contents of the stack along with variable names, for debug purposes

### Simple example program
```
push 1
push 2
add
```

This ends with the value `3` on the stack.

### roll command example
```
push 7
push 6
push 5
push 4
push 3
push 2
push 1

push 3
push 1

roll
```

This yields a stack that, from top to bottom looks like: `7, 6, 5, 4, 1, 3, 2`


## Test Fixtures

Integration-style tests in `src/main.rs` use fixture files under `tests/fixtures/`:
- `add.txt` — simple addition
- `roll.txt` — roll operation
- `mandelbrot_complex.txt` — fixed-point complex number arithmetic (FACTOR=100) for Mandelbrot set computation
