---
shaping: true
---

# Mandelbrot Piet — Shaping

## Requirements (R)

| ID | Requirement | Status |
|----|-------------|--------|
| R0 | Print Mandelbrot ASCII art to stdout matching the Ruby prototype (y: 100→-100 step -5, x: -200→50 step 3, 4 iterations, FACTOR=100) | Core goal |
| R1 | Phase 1 program runs correctly in the existing Rust interpreter | Must-have |
| R2 | Phase 1 program uses only operations that have Piet equivalents; `branch` is permitted as the text-program proxy for visual Piet's spatial loop-back paths; `debug_stack` must not appear in the final program | Must-have |
| R3 | Phase 2 produces a valid, runnable visual Piet image | Core goal |
| R4 | The mandelbrot_complex.txt computation logic is reused rather than rewritten from scratch | Nice-to-have |
| R5 | Intermediate artifacts are testable (unit-level tests for sub-computations) | Nice-to-have |

---

## Phase 1: Text Program

### Constraint: `branch` is a goto, not an indirect jump

The `branch` instruction jumps to a hardcoded line number (`comm.value`). There is no "pop return address and jump" instruction. This means traditional subroutine calls are not possible without additional interpreter support.

`branch` is the correct text-program equivalent of a visual Piet loop: in visual Piet, a loop is a spatial path that routes the DP back to a previously-visited region. In the text program, `branch` plays that role.

**Consequence:** The mandelbrot computation must be inlined (or structured as a single region jumped *into*, with a hardcoded return site). No true subroutines.

---

### A: Fully Inlined Flat Program

One `.txt` file. The outer y-loop and inner x-loop manage the stack. The mandelbrot computation (adapted from `mandelbrot_complex.txt`) is inlined in the loop body.

| Part | Mechanism |
|------|-----------|
| A1 | **Outer y-loop** — initialize y=100, loop until y < -100, decrement by 5 each iteration |
| A2 | **Inner x-loop** — for each y, initialize x=-200, loop until x > 50, increment by 3 |
| A3 | **Mandelbrot compute** — adapted `mandelbrot_complex.txt` inline, with loop counter reset each entry |
| A4 | **Point output** — compare result < 4*FACTOR^2 (40000); branch to print `*` (42) or ` ` (32) via `out_char` |
| A5 | **Row end** — after inner loop exhausted, print newline (10) via `out_char`, resume outer loop |
| A6 | **Stack discipline** — at each loop entry the stack holds: `[y, x, inner_loop_ctr, ...]`; roll used to access/restore values around mandelbrot computation |

**Stack layout entering mandelbrot:**
```
top → [mandelbrot_iters(4), zr(0), zi(0), x, y, x_loop_ctr, x, y, y_loop_ctr]
```
(x and y are duplicated on entry so they survive the computation)

### B: Text Preprocessor (Macro Expansion)

Write a small Rust tool or script that concatenates/includes sub-files and resolves label-based `branch` targets, so the mandelbrot code can live in its own file.

| Part | Mechanism |
|------|-----------|
| B1 | **Include directive** — `#include mandelbrot_complex.txt` in the main program file |
| B2 | **Label-based branch** — `branch :loop_start` resolved to line numbers at assemble time |
| B3 | **Assembler** — Rust binary that reads the source, resolves includes/labels, writes a flat `.txt` for the interpreter |
| B4 | **Main loop file** — outer/inner loop + output logic, referencing mandelbrot as an include |

---

## Fit Check: Phase 1

| Req | Requirement | Status | A | B |
|-----|-------------|--------|---|---|
| R0 | Print Mandelbrot ASCII art matching Ruby prototype | Core goal | ✅ | ✅ |
| R1 | Runs correctly in existing Rust interpreter | Must-have | ✅ | ✅ |
| R2 | Only true-Piet operations in hot path | Must-have | ✅ | ✅ |
| R4 | Reuses mandelbrot_complex.txt logic | Nice-to-have | ✅ | ✅ |
| R5 | Intermediate artifacts are testable | Nice-to-have | ❌ | ✅ |

**Notes:**
- A fails R5: with everything inlined and hardcoded line numbers, sub-unit testing is harder (though integration tests still work)
- B adds significant tooling overhead for a one-off program; label resolution would be a new feature in the interpreter or a separate tool

**Selected shape: A** — Simpler, no new tooling needed. The mandelbrot computation is already tested via `test_mandelbrot_complex`. Integration test of the full program output suffices.

---

## Phase 1 Detailed Structure (Detail A)

### Stack conventions

The interpreter's stack grows rightward (rightmost = top). The mandelbrot compute segment from `mandelbrot_complex.txt` **expects** `a` (x) and `b` (y) already on the stack when it starts.

### Program skeleton (line regions, not actual line numbers yet)

```
Region 0 — Init outer loop
    push 100        # y_start
    [y_loop_top:]

Region 1 — Init inner loop
    push -200       # x_start
    [x_loop_top:]

Region 2 — Mandelbrot compute
    # dup x and y so they survive
    # push zr=0, zi=0, iter=4
    # [4-iteration loop]
    # result: magnitude^2 on stack (a_acc^2/F + b_acc^2/F)
    # clean stack → [mag2, x, y, x_loop_ctr, y_loop_ctr]

Region 3 — Output character
    push 40000      # 4 * 100 * 100
    >               # mag2 > 40000? (i.e., NOT in set)
    branch [print_space]
    push 42         # '*'
    out_char
    branch [after_char]
    [print_space:]
    push 32         # ' '
    out_char
    [after_char:]

Region 4 — Inner loop step
    # x += 3
    push 3
    +
    # check x <= 50
    dup
    push 50
    >               # x > 50?
    not             # x <= 50?
    branch [x_loop_top]

Region 5 — Row end
    pop             # discard x
    push 10         # newline
    out_char

Region 6 — Outer loop step
    # y -= 5
    push 5
    -
    # check y >= -100
    dup
    push -100
    >               # y > -100?  (use > since we want y >= -100 → continue)
    branch [y_loop_top]
```

**Key challenge:** The mandelbrot_complex.txt currently uses a hardcoded loop counter of 10 (line `push 10 loop_counter`) and `branch 10` — these need to be adjusted:
- Loop counter should start at 4 (4 iterations), not 10
- The branch target must be the correct line number in the assembled file

---

## Phase 2: Visual Piet

### Background

Visual Piet uses colored pixel blocks ("codels") navigated by a Direction Pointer (DP) and Codel Chooser (CC). Instructions are encoded in color transitions between adjacent codel regions. There is no line-number addressing — control flow is purely spatial.

### Shapes for Phase 2

**X: Manual image construction**
Hand-draw the Piet image using a paint tool or pixel editor, following a known "linear strip" layout (a snake-like path through the image).

| Part | Mechanism |
|------|-----------|
| X1 | Map each text instruction to its Piet color transition |
| X2 | Lay out codels in a snake/zigzag path |
| X3 | Handle loops with backtracking paths in the 2D grid |

**Y: Code generator (text → Piet image)**
Write a Rust (or Ruby) tool that reads the text program and emits a Piet-compatible PNG or PPM image. Uses a "linear road" layout where the snake path is generated automatically.

| Part | Mechanism |
|------|-----------|
| Y1 | **Instruction map** — table of (previous_color, instruction) → next_color |
| Y2 | **Path generator** — emit a horizontal snake layout, N pixels wide |
| Y3 | **Branch layout** — for each `branch`, emit a colored "ramp" that detours the DP around or back |
| Y4 | **PNG/PPM writer** — output the codel grid as an image file |

**Z: Transpile to existing Piet programs / use a known Mandelbrot Piet**
Adapt or reference an existing Piet Mandelbrot implementation.

---

## Fit Check: Phase 2

| Req | Requirement | Status | X | Y | Z |
|-----|-------------|--------|---|---|---|
| R0 | Print Mandelbrot ASCII art | Core goal | ✅ | ✅ | ✅ |
| R3 | Produces valid, runnable visual Piet image | Core goal | ✅ | ✅ | ✅ |
| R2 | Only true-Piet operations | Must-have | ✅ | ✅ | ✅ |
| R5 | Intermediate artifacts are testable | Nice-to-have | ❌ | ✅ | ❌ |

**Notes:**
- X is high-effort and error-prone; branch handling in 2D is extremely hard to do by hand
- Z sidesteps the learning/building goal
- Y is the most tractable: once the text program works, a mechanical translation is automatable

**Recommended shape: Y** — but Y3 (branch layout in 2D Piet) is flagged ⚠️: the mechanism for encoding backward jumps/loops in a linear Piet layout is not fully understood yet.

---

## Open Questions / Spikes Needed

### Spike 1: Branch encoding in linear-strip Piet

| # | Question |
|---|----------|
| S1-Q1 | How do existing Piet programs implement loops in the 2D grid? |
| S1-Q2 | Can a purely linear (1D snake) path encode backwards branches, or does the layout need 2D detour paths? |
| S1-Q3 | What is the maximum program size (number of codels) for a Mandelbrot program, and what image dimensions are needed? |

**Acceptance:** We can describe a concrete layout strategy for encoding the x/y nested loops in a Piet image.

---

## Recommended Next Steps

1. **Build and test Phase 1** — Write `programs/mandelbrot.txt` (the full flat program), run it via `cargo run -- programs/mandelbrot.txt`, verify output matches `prototype.rb`.
2. **Spike S1** — Investigate how loops are encoded in visual Piet before committing to Phase 2 shape.
3. **Then decide Phase 2 approach** — With S1 answered, finalize Y vs manual.
