use std::fs::File;
use std::io::{self, BufRead};

// https://www.dangermouse.net/esoteric/piet.html

#[derive(PartialEq, Debug)]
pub enum CodelChooser {
    Left,
    Right,
}

#[derive(PartialEq, Debug)]
pub enum DirectionPointer {
    Right,
    Down,
    Left,
    Up,
}

#[derive(Clone)]
pub enum CommandType {
    Push,
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
    // not real piet commands, useful for this pseudocode
    Branch,
    DebugStack,
    OutLabel,
    NoOp,
    ResetColor,
}

#[derive(Clone)]
pub struct Command {
    pub action: CommandType,
    pub value: i32,
    pub label: String,
    pub source: String,
}

impl Command {
    pub fn clean_line(line: &str) -> &str {
        let line = line.trim();

        if let Some(comment) = line.find('#') {
            return &line[0..comment].trim();
        } else {
            return line;
        }
    }
    pub fn parse(line: &str) -> Command {
        let source = line.to_string();
        let line = Command::clean_line(line);
        if line == "" {
            // blank line is also a noop, for easy goto
            return Command {
                action: CommandType::NoOp,
                value: -1,
                label: "".to_string(),
                source
            };
        }
        let split: Vec<&str> = line.split(' ').collect();
        assert!(split.len() > 0);
        let cmd = split[0].to_ascii_lowercase();
        Command {
            action: {
                match cmd.as_str() {
                    "push" => CommandType::Push,
                    "pop" => CommandType::Pop,
                    "add" | "+" => CommandType::Add,
                    "subtract" | "-" => CommandType::Subtract,
                    "multiply" | "*" => CommandType::Multiply,
                    "divide" | "/" => CommandType::Divide,
                    "mod" | "%" => CommandType::Mod,
                    "not" => CommandType::Not,
                    "greater" | ">" => CommandType::Greater,
                    "pointer" => CommandType::Pointer,
                    "switch" => CommandType::Switch,
                    "duplicate" | "dup" => CommandType::Duplicate,
                    "roll" => CommandType::Roll,
                    "in_number" => CommandType::InNumber,
                    "in_char" => CommandType::InChar,
                    "out_number" => CommandType::OutNumber,
                    "out_char" => CommandType::OutChar,
                    "branch" => CommandType::Branch,
                    "debug_stack" => CommandType::DebugStack,
                    "out_label" => CommandType::OutLabel,
                    "noop" | "#" => CommandType::NoOp,
                    "reset_color" => CommandType::ResetColor,
                    _ => panic!("bad command: {}", split[0])
                }
            },
            value: if split.len() > 1 && split[0] != "#" {
                split[1].parse().unwrap_or(-1)
            } else {
                -1
            },
            label: match cmd.as_str() {
                "push" => split.get(2).unwrap_or(&"").to_string(),
                "add" | "+" |
                "subtract" | "-" |
                "multiply" | "*" |
                "divide" | "/" |
                "mod" | "%" |
                "not" |
                "greater" | ">" |
                "duplicate" | "dup" |
                "in_number" |
                "in_char"  => split.get(1).unwrap_or(&"").to_string(),
                _ => "".to_string()
            },
            source

        }
    }
}

pub fn read_file(file_path: &str) -> Vec<Command> {
    println!("Openning {}", file_path);
    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);

    let mut ret: Vec<Command> = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();

        ret.push(Command::parse(&line))
    }

    return ret
}
