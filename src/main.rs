use std::fs::{read, File};
use std::io::{self, BufRead};
use std::num;
use std::path::Component;

// https://www.dangermouse.net/esoteric/piet.html

enum CodelChooser {
    Left,
    Right,
}

enum DirectionPointer {
    Right,
    Down,
    Left,
    Up,
}

enum CommandType {
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
    NoOp,
}

struct Command {
    action: CommandType,
    value: i32,
    label: String,
}

impl Command {
    fn clean_line(line: &str) -> &str {
        let line = line.trim();

        if let Some(comment) = line.find('#') {
            return &line[0..comment].trim();
        } else {
            return line;
        }
    }
    pub fn parse(line: &str) -> Command {
        let line = Command::clean_line(line);
        if line == "" {
            // blank line is also a noop, for easy goto
            return Command {
                action: CommandType::NoOp,
                value: -1,
                label: "".to_string()
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
                    "noop" | "#" => CommandType::NoOp,
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
            }

        }
    }
}

fn main() {
    let commands = read_file("program.txt");

    run_code(commands);
}

fn run_code(commands:Vec<Command>) -> (Vec<i32>, Vec<String>, DirectionPointer, CodelChooser) {
    let mut stack: Vec<i32> = Vec::new();
    let mut labels: Vec<&str> = Vec::new();
    let mut dp: DirectionPointer = DirectionPointer::Right;
    let mut cc: CodelChooser = CodelChooser::Left;

    let mut command_num: usize = 0;

    while command_num < commands.len() {
        let comm = &commands[command_num];
        command_num += 1;

        match comm.action {
            CommandType::Push => {
                stack.push(comm.value);
                labels.push(&comm.label);
            },
            CommandType::Pop => {
                stack.pop();
                labels.pop();
            },
            CommandType::Add => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                labels.pop();
                labels.pop();
                stack.push(a + b);
                labels.push(&comm.label);
            },
            CommandType::Subtract => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                labels.pop();
                labels.pop();
                stack.push(b - a);
                labels.push(&comm.label);
            },
            CommandType::Multiply => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                labels.pop();
                labels.pop();
                stack.push(a * b);
                labels.push(&comm.label);
            },
            CommandType::Divide => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                labels.pop();
                labels.pop();
                stack.push(b / a);
                labels.push(&comm.label);
            },
            CommandType::Mod => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                labels.pop();
                labels.pop();
                stack.push(b % a);
                labels.push(&comm.label);
            },
            CommandType::Not => {
                let x = stack.pop().unwrap();
                labels.pop();
                stack.push(if x != 0 {
                    0
                } else {
                    1
                });
                labels.push(&comm.label);
            }
            CommandType::Greater => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                labels.pop();
                labels.pop();
                stack.push(if b > a {
                    1
                } else {
                    0
                });
                labels.push(&comm.label);
            },
            CommandType::Pointer => todo!(),
            CommandType::Switch => todo!(),
            CommandType::Duplicate => {
                let x = stack.pop().unwrap();
                // Don't pop the label, because we put this right back
                stack.push(x);
                stack.push(x);
                labels.push(&comm.label);
            },
            CommandType::Roll => {
                // https://piet.forumotion.com/t7-roll-implementation
                let num_rolls = stack.pop().unwrap();
                let depth = stack.pop().unwrap() as usize;
                labels.pop();
                labels.pop();

                let end_ptr = stack.len()-1;
                for _ in 0..num_rolls {
                    for i in 0..depth-1 {
                        stack.swap(end_ptr-i, end_ptr-i-1);
                        labels.swap(end_ptr-i, end_ptr-i-1);
                    }
                }

                // todo: handle negative num_rolls
            },
            CommandType::InNumber => todo!(),
            CommandType::InChar => todo!(),
            CommandType::OutNumber => {
                println!("{}", stack.pop().unwrap());
                labels.pop();
            },
            CommandType::OutChar => {
                println!("{}", char::from_u32(stack.pop().unwrap() as u32).unwrap());
                labels.pop();
            },
            CommandType::Branch => {
                // TODO: make this conditional
                command_num = comm.value as usize;
            },
            CommandType::DebugStack => {
                println!("Stack: ");
                
                for i in (0..stack.len()).rev() {
                    let num = stack[i].to_string();
                    print!("{}", num);
                    for _ in 0..(10 - num.len()) {
                        print!(" ");
                    }
                    println!("{}", labels[i]);
                }
            },
            CommandType::NoOp => {
                // Do nothing
            }
        }
    }

    let ret_labels: Vec<String> = labels.iter().map(|x| x.to_string()).collect();

    (stack, ret_labels, dp, cc)
}

fn read_file(file_path: &str) -> Vec<Command> {
    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);

    let mut ret: Vec<Command> = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();

        ret.push(Command::parse(&line))
    }

    return ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll() {
        let cmds = read_file("tests/fixtures/roll.txt");
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![7, 6, 5, 4, 1, 3, 2]);
        assert_eq!(labels, vec!["a", "b", "c", "d", "g", "e", "f"]);
    }

    #[test]
    fn test_add() {
        let cmds = read_file("tests/fixtures/add.txt");
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![3]);
        assert_eq!(labels, vec![""]);
    }

    #[test]
    fn test_sub() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("push 4"),
            Command::parse("- answer")
        ];
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![-1]);
        assert_eq!(labels, vec!["answer"]);
    }

    #[test]
    fn test_multiply() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("push 4"),
            Command::parse("push 5"),
            Command::parse("* answer")
        ];
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![3, 20]);
        assert_eq!(labels, vec!["", "answer"]);
    }

    #[test]
    fn test_divide() {
        let cmds = vec![
            Command::parse("push 11"),
            Command::parse("push 2"),
            Command::parse("/")
        ];
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![5]);
        assert_eq!(labels, vec![""]);
    }

    #[test]
    fn test_not() {
        let cmds = vec![
            Command::parse("push 3 a1"),
            Command::parse("not a2"),
            Command::parse("push 0 b1"),
            Command::parse("not b2"),
            Command::parse("push 1 b1"),
            Command::parse("not c2")
        ];
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![0, 1, 0]);
        assert_eq!(labels, vec!["a2", "b2", "c2"]);
    }

    #[test]
    fn test_greater() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("push 0"),
            Command::parse("> answer"),
        ];
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![1]);
        assert_eq!(labels, vec!["answer"]);

        let cmds = vec![
            Command::parse("push 1 a"),
            Command::parse("push 6 b"),
            Command::parse("> answer")
        ];
        let (stack, labels, _, _) = run_code(cmds);
        assert_eq!(stack, vec![0]);
        assert_eq!(labels, vec!["answer"]);
    }
}