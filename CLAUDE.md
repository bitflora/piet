# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust interpreter for a text-based variant of the [Piet esoteric programming language](https://www.dangermouse.net/esoteric/piet.html). The original Piet uses images with colored pixels; this project uses plain text files for easier prototyping before converting to the visual format.

## Commands

```bash
cargo build              # Build
cargo run                # Run default program (program.txt)
cargo run -- myfile.txt  # Run a specific program file
cargo test               # Run all tests
cargo test -- --nocapture  # Run tests with stdout visible
```

## Architecture

Everything lives in `src/main.rs`. The interpreter is intentionally simple with no external dependencies.

**Execution flow:**
1. `main()` reads the program file path from args, defaults to `program.txt`
2. `read_file()` parses each line into a `Command` via `Command::parse()`
3. `run_code(commands, debug)` executes the command list sequentially, returns final state

**Key types:**
- `Command` — one instruction with `action: CommandType`, `value: i32`, optional `label`, and `source` text
- `CommandType` — 25 variants covering stack ops, arithmetic, logic, I/O, control flow, and navigation
- Runtime state: `stack: Vec<i32>`, `labels: Vec<&str>`, `dp: DirectionPointer`, `cc: CodelChooser`, and a `program_counter: usize`

**`Branch` semantics:** pops top of stack; if non-zero, jumps to the line number given as the command's value (0-indexed).

**`Roll` semantics:** pops depth and count, then rotates the top `depth` elements of the stack `count` times. This mirrors the original Piet `roll` spec.

**`Pointer`/`Switch`:** rotate `DirectionPointer` or toggle `CodelChooser` by popping the top of stack. These mirror visual Piet navigation primitives.

## Test Fixtures

Integration-style tests in `src/main.rs` use fixture files under `tests/fixtures/`:
- `add.txt` — simple addition
- `roll.txt` — roll operation
- `mandelbrot_complex.txt` — fixed-point complex number arithmetic (FACTOR=100) for Mandelbrot set computation
