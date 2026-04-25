use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use gif::{Encoder, Frame, Repeat};
use piet::{Command, CommandType, read_file};

// Piet color palette. Index = lightness * 6 + hue
// hue: 0=red 1=yellow 2=green 3=cyan 4=blue 5=magenta
// lightness: 0=light 1=normal 2=dark
// Padded to 32 entries for GIF color table requirements.
#[rustfmt::skip]
const PALETTE: &[u8] = &[
    // Light (lightness=0): red, yellow, green, cyan, blue, magenta
    0xFF, 0xC0, 0xC0,  // 0:  light red
    0xFF, 0xFF, 0xC0,  // 1:  light yellow
    0xC0, 0xFF, 0xC0,  // 2:  light green
    0xC0, 0xFF, 0xFF,  // 3:  light cyan
    0xC0, 0xC0, 0xFF,  // 4:  light blue
    0xFF, 0xC0, 0xFF,  // 5:  light magenta
    // Normal (lightness=1): red, yellow, green, cyan, blue, magenta
    0xFF, 0x00, 0x00,  // 6:  normal red
    0xFF, 0xFF, 0x00,  // 7:  normal yellow
    0x00, 0xFF, 0x00,  // 8:  normal green
    0x00, 0xFF, 0xFF,  // 9:  normal cyan
    0x00, 0x00, 0xFF,  // 10: normal blue
    0xFF, 0x00, 0xFF,  // 11: normal magenta
    // Dark (lightness=2): red, yellow, green, cyan, blue, magenta
    0xC0, 0x00, 0x00,  // 12: dark red
    0xC0, 0xC0, 0x00,  // 13: dark yellow
    0x00, 0xC0, 0x00,  // 14: dark green
    0x00, 0xC0, 0xC0,  // 15: dark cyan
    0x00, 0x00, 0xC0,  // 16: dark blue
    0xC0, 0x00, 0xC0,  // 17: dark magenta
    // Black
    0x00, 0x00, 0x00,  // 18: black
    // White (used for padding in vertical layout)
    0xFF, 0xFF, 0xFF,  // 19: white
    // Padding to 32 entries (required by GIF color table spec)
    0x00, 0x00, 0x00,  // 20-31: unused (black)
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
];

const BLACK_IDX: u8 = 18;
const WHITE_IDX: u8 = 19;

fn color_index(hue: u8, lightness: u8) -> u8 {
    lightness * 6 + hue
}

// Low-level Piet operations with encoded block sizes.
// Push(n) means: emit a block of size n; the push command pushes n.
enum PietOp {
    Push(u32),
    Pop,
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Not,
    Greater,
    Pointer,
    Switch,
    Duplicate,
    Roll,
    InNumber,
    InChar,
    OutNumber,
    OutChar,
    White,
}

// Returns (hue_delta, lightness_delta) for each command per the Piet spec table.
fn command_delta(op: &PietOp) -> (u8, u8) {
    match op {
        PietOp::Push(_)    => (0, 1),
        PietOp::Pop        => (0, 2),
        PietOp::Add        => (1, 0),
        PietOp::Subtract   => (1, 1),
        PietOp::Multiply   => (1, 2),
        PietOp::Divide     => (2, 0),
        PietOp::Mod        => (2, 1),
        PietOp::Not        => (2, 2),
        PietOp::Greater    => (3, 0),
        PietOp::Pointer    => (3, 1),
        PietOp::Switch     => (3, 2),
        PietOp::Duplicate  => (4, 0),
        PietOp::Roll       => (4, 1),
        PietOp::InNumber   => (4, 2),
        PietOp::InChar     => (5, 0),
        PietOp::OutNumber  => (5, 1),
        PietOp::OutChar    => (5, 2),
        PietOp::White      => unreachable!("White is handled before command_delta"),
    }
}

struct PietBlock {
    color_idx: u8,
    size: u32,
}

// Translate filtered Pietxt commands into a flat sequence of Piet operations.
// push 0   → push 1; not         (result: 0)
// push -N  → push 1; not; push N; subtract  (result: 0 - N = -N)
// push N>0 → push N              (direct)
// branch X → not; not; pointer   (normalize TOS to 0/1, rotate DP by that amount)
//   ... but if the immediately preceding real command was `greater`, skip not;not since
//   greater already produces exactly 0 or 1: branch X → pointer
fn expand_commands(commands: Vec<Command>) -> Vec<PietOp> {
    let mut ops = Vec::new();
    let mut prev_was_greater = false;
    for cmd in commands {
        match cmd.action {
            CommandType::Push => {
                let v = cmd.value;
                if v > 0 {
                    ops.push(PietOp::Push(v as u32));
                } else if v == 0 {
                    ops.push(PietOp::Push(1));
                    ops.push(PietOp::Not);
                } else {
                    // push 1; not → 0; push |v|; subtract → 0 - |v| = v
                    ops.push(PietOp::Push(1));
                    ops.push(PietOp::Not);
                    ops.push(PietOp::Push(v.unsigned_abs()));
                    ops.push(PietOp::Subtract);
                }
                prev_was_greater = false;
            },
            CommandType::Pop        => { ops.push(PietOp::Pop);       prev_was_greater = false; },
            CommandType::Add        => { ops.push(PietOp::Add);        prev_was_greater = false; },
            CommandType::Subtract   => { ops.push(PietOp::Subtract);   prev_was_greater = false; },
            CommandType::Multiply   => { ops.push(PietOp::Multiply);   prev_was_greater = false; },
            CommandType::Divide     => { ops.push(PietOp::Divide);     prev_was_greater = false; },
            CommandType::Mod        => { ops.push(PietOp::Mod);        prev_was_greater = false; },
            CommandType::Not        => { ops.push(PietOp::Not);        prev_was_greater = false; },
            CommandType::Greater    => { ops.push(PietOp::Greater);    prev_was_greater = true;  },
            CommandType::Pointer    => { ops.push(PietOp::Pointer);    prev_was_greater = false; },
            CommandType::Switch     => { ops.push(PietOp::Switch);     prev_was_greater = false; },
            CommandType::Duplicate  => { ops.push(PietOp::Duplicate);  prev_was_greater = false; },
            CommandType::Roll       => { ops.push(PietOp::Roll);       prev_was_greater = false; },
            CommandType::InNumber   => { ops.push(PietOp::InNumber);   prev_was_greater = false; },
            CommandType::InChar     => { ops.push(PietOp::InChar);     prev_was_greater = false; },
            CommandType::OutNumber  => { ops.push(PietOp::OutNumber);  prev_was_greater = false; },
            CommandType::OutChar    => { ops.push(PietOp::OutChar);    prev_was_greater = false; },
            CommandType::Branch => {
                // When prev was `greater`, its output is already 0 or 1 — skip not;not.
                if !prev_was_greater {
                    ops.push(PietOp::Not);
                    ops.push(PietOp::Not);
                }
                ops.push(PietOp::Pointer);
                prev_was_greater = false;
            },
            CommandType::ResetColor => { ops.push(PietOp::White); prev_was_greater = false; },
            // Non-emitting: leave prev_was_greater unchanged so NoOp etc. don't
            // break a greater→branch adjacency.
            CommandType::DebugStack | CommandType::OutLabel | CommandType::NoOp => {},
        }
    }
    ops
}

// Assign colors to each block. Crossing block[i] → block[i+1] executes op[i].
// The push value for a push op equals block[i].size (codels in the exited block).
//
// Block layout: B0 (initial, no cmd) | B1..Bn (one per op) | B_black (terminator)
// Total: n+2 blocks for n ops.
fn assign_colors(ops: &[PietOp]) -> Vec<PietBlock> {
    let mut blocks = Vec::new();
    let mut hue: u8 = 0;       // start: light red
    let mut lightness: u8 = 0;

    for op in ops {
        if matches!(op, PietOp::White) {
            blocks.push(PietBlock { color_idx: WHITE_IDX, size: 1 });
            hue = 0;
            lightness = 0;
            continue;
        }
        let size = match op {
            PietOp::Push(n) => *n,
            _ => 1,
        };
        let (dh, dl) = command_delta(op);
        blocks.push(PietBlock { color_idx: color_index(hue, lightness), size });
        hue = (hue + dh) % 6;
        lightness = (lightness + dl) % 3;
    }

    // Final colored block (IP enters here after last command executes).
    blocks.push(PietBlock { color_idx: color_index(hue, lightness), size: 1 });
    // Black terminator — IP cannot enter, triggers program termination.
    blocks.push(PietBlock { color_idx: BLACK_IDX, size: 1 });

    blocks
}

fn render_gif_to_writer<W: Write>(blocks: &[PietBlock], codel_size: u32, vertical: bool, writer: W) {
    let (width, height, pixels) = if vertical {
        let max_size: u32 = blocks.iter().map(|b| b.size).max().unwrap_or(1);
        let w = u16::try_from(blocks.len() as u32 * codel_size)
            .expect("image too wide: too many blocks");
        let h = u16::try_from(max_size * codel_size)
            .expect("image too tall: reduce push values or codel size");

        // Build row by row. Each block occupies one codel column; its color fills
        // the top block.size codel rows, white fills the remainder.
        let mut px: Vec<u8> = Vec::with_capacity(w as usize * h as usize);
        for pixel_row in 0..h as u32 {
            let codel_row = pixel_row / codel_size;
            for block in blocks {
                let color = if codel_row < block.size { block.color_idx } else { WHITE_IDX };
                for _ in 0..codel_size {
                    px.push(color);
                }
            }
        }
        (w, h, px)
    } else {
        let total_codels: u32 = blocks.iter().map(|b| b.size).sum();
        let w = u16::try_from(total_codels * codel_size)
            .expect("image too wide: reduce push values or codel size");
        let h = u16::try_from(codel_size)
            .expect("codel size too large");

        // Build a single pixel row (each block is size*codel_size pixels wide).
        let mut row: Vec<u8> = Vec::with_capacity(w as usize);
        for block in blocks {
            let pixel_width = block.size * codel_size;
            for _ in 0..pixel_width {
                row.push(block.color_idx);
            }
        }

        // Tile the row vertically to produce a codel_size-tall image.
        let mut px: Vec<u8> = Vec::with_capacity(w as usize * h as usize);
        for _ in 0..h {
            px.extend_from_slice(&row);
        }
        (w, h, px)
    };

    let mut encoder = Encoder::new(writer, width, height, PALETTE)
        .expect("failed to create GIF encoder");
    encoder.set_repeat(Repeat::Finite(0)).expect("failed to set repeat");

    let mut frame = Frame::default();
    frame.width = width;
    frame.height = height;
    frame.buffer = Cow::Owned(pixels);
    encoder.write_frame(&frame).expect("failed to write GIF frame");
}

fn render_gif(blocks: &[PietBlock], codel_size: u32, vertical: bool, output_path: &str) {
    let output = File::create(output_path)
        .unwrap_or_else(|e| panic!("cannot create {}: {}", output_path, e));
    render_gif_to_writer(blocks, codel_size, vertical, output);
}

#[cfg(test)]
fn compile_to_gif_bytes(commands: Vec<Command>, codel_size: u32) -> Vec<u8> {
    let ops = expand_commands(commands);
    let blocks = assign_colors(&ops);
    let mut buf = Vec::new();
    render_gif_to_writer(&blocks, codel_size, false, &mut buf);
    buf
}

fn print_usage(prog: &str) {
    eprintln!("Usage: {} <file> [--start N] [--end N] [-o output.gif] [--codel-size N] [--vertical]", prog);
    eprintln!();
    eprintln!("  --start N       First line to compile (0-indexed, inclusive)");
    eprintln!("  --end N         Last line to compile (0-indexed, inclusive)");
    eprintln!("  -o / --output   Output GIF path (default: <input>.gif)");
    eprintln!("  --codel-size N  Pixels per codel (default: 1)");
    eprintln!("  --vertical      Stack commands top-to-bottom; white fills unused space");
}

#[cfg(test)]
mod tests {
    use super::*;
    use piet::read_file;

    // codel_size=1 keeps golden files small; must match the bootstrap commands
    fn compile_fixture(name: &str) -> Vec<u8> {
        let commands = read_file(&format!("tests/fixtures/pietc/{}.txt", name));
        compile_to_gif_bytes(commands, 1)
    }

    fn expected_bytes(name: &str) -> Vec<u8> {
        let path = format!("tests/fixtures/pietc/{}.gif", name);
        std::fs::read(&path)
            .unwrap_or_else(|e| panic!("Missing golden file {}.gif — run bootstrap commands: {}", name, e))
    }

    #[test]
    fn test_push_5() {
        assert_eq!(compile_fixture("push_5"), expected_bytes("push_5"));
    }

    #[test]
    fn test_push_add() {
        assert_eq!(compile_fixture("push_add"), expected_bytes("push_add"));
    }

    #[test]
    fn test_push_zero() {
        assert_eq!(compile_fixture("push_zero"), expected_bytes("push_zero"));
    }

    #[test]
    fn test_push_negative() {
        assert_eq!(compile_fixture("push_negative"), expected_bytes("push_negative"));
    }

    #[test]
    fn test_simple_loop_structure() {
        // Validates branch compilation without depending on a golden GIF.
        let commands = read_file("tests/fixtures/pietc/simple_loop.txt");
        let ops = expand_commands(commands);

        // push(3),dup,out_number,push(1),subtract,dup,not,not,pointer,pop = 10 ops
        assert_eq!(ops.len(), 10);
        assert!(matches!(ops[6], PietOp::Not));
        assert!(matches!(ops[7], PietOp::Not));
        assert!(matches!(ops[8], PietOp::Pointer));
    }

    #[test]
    fn test_simple_loop_gif() {
        assert_eq!(compile_fixture("simple_loop"), expected_bytes("simple_loop"));
    }

    #[test]
    fn test_reset_color_restarts_color_sequence() {
        // reset_color emits white then resets hue/lightness to 0 (light red).
        // Ops: [Add, White, Push(3)]
        let ops = vec![PietOp::Add, PietOp::White, PietOp::Push(3)];
        let blocks = assign_colors(&ops);

        // Expected layout: [Add-from, white, Push(3)-from, FINAL_TO, black]
        assert_eq!(blocks.len(), 5);
        assert_eq!(blocks[0].color_idx, color_index(0, 0)); // light red (initial)
        assert_eq!(blocks[1].color_idx, WHITE_IDX);
        assert_eq!(blocks[2].color_idx, color_index(0, 0)); // light red (reset)
        assert_eq!(blocks[2].size, 3);                       // Push(3) encodes value in size
        assert_eq!(blocks[3].color_idx, color_index(0, 1)); // normal red (after Push delta dl=1)
        assert_eq!(blocks[4].color_idx, BLACK_IDX);
    }

    #[test]
    fn test_branch_after_greater_skips_not_not() {
        // When `branch` immediately follows `greater`, not;not is omitted.
        let commands = read_file("tests/fixtures/pietc/greater_loop.txt");
        let ops = expand_commands(commands);

        // push(3),dup,out_number,push(1),subtract,dup,push(1),not,greater,pointer,pop = 11 ops
        // (push 0 → push 1; not; no not;not before pointer since prev was greater)
        assert_eq!(ops.len(), 11);
        assert!(matches!(ops[8], PietOp::Greater));
        assert!(matches!(ops[9], PietOp::Pointer));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog = &args[0];

    let mut input_file: Option<String> = None;
    let mut output_file: Option<String> = None;
    let mut start_line: Option<usize> = None;
    let mut end_line: Option<usize> = None;
    let mut codel_size: u32 = 1;
    let mut vertical: bool = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--start" => {
                i += 1;
                start_line = Some(args[i].parse().expect("--start requires a non-negative integer"));
            },
            "--end" => {
                i += 1;
                end_line = Some(args[i].parse().expect("--end requires a non-negative integer"));
            },
            "-o" | "--output" => {
                i += 1;
                output_file = Some(args[i].clone());
            },
            "--codel-size" => {
                i += 1;
                codel_size = args[i].parse().expect("--codel-size requires a positive integer");
            },
            "--vertical" => {
                vertical = true;
            },
            arg if !arg.starts_with('-') => {
                input_file = Some(arg.to_string());
            },
            arg => {
                eprintln!("Unknown argument: {}", arg);
                print_usage(prog);
                std::process::exit(1);
            },
        }
        i += 1;
    }

    let input_path = match input_file {
        Some(p) => p,
        None => {
            eprintln!("Error: input file required");
            print_usage(prog);
            std::process::exit(1);
        }
    };

    let output_path = output_file.unwrap_or_else(|| {
        Path::new(&input_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            + ".gif"
    });

    let all_commands = read_file(&input_path);
    let start = start_line.unwrap_or(0);
    let commands: Vec<Command> = if let Some(end) = end_line {
        all_commands.into_iter().skip(start).take(end - start + 1).collect()
    } else {
        all_commands.into_iter().skip(start).collect()
    };

    let ops = expand_commands(commands);
    let blocks = assign_colors(&ops);
    let total_codels: u32 = blocks.iter().map(|b| b.size).sum();

    println!("Compiling {} ops → {} codels → {}", ops.len(), total_codels, output_path);
    render_gif(&blocks, codel_size, vertical, &output_path);
    println!("Wrote {}", output_path);
}
