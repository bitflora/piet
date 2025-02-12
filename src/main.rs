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
    value: i32
}

impl Command {
    fn parse(line: &str) -> Command {
        if line == "" {
            // blank line is also a noop, for easy goto
            return Command {
                action: CommandType::NoOp,
                value: -1
            };
        }
        let split: Vec<&str> = line.split(' ').collect();
        assert!(split.len() > 0);
        Command {
            action: {
                match split[0].to_ascii_lowercase().as_str() {
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
            value: if split.len() > 1 {
                split[1].parse().unwrap()
            } else {
                -1
            }

        }
    }
}

fn main() {
    let commands = read_file("program.txt");

    run_code(commands);
}

fn run_code(commands:Vec<Command>) -> (Vec<i32>, DirectionPointer, CodelChooser) {
    let mut stack: Vec<i32> = Vec::new();
    let mut dp: DirectionPointer = DirectionPointer::Right;
    let mut cc: CodelChooser = CodelChooser::Left;

    let mut command_num: usize = 0;

    while command_num < commands.len() {
        let comm = &commands[command_num];
        command_num += 1;

        match comm.action {
            CommandType::Push => {
                stack.push(comm.value);
            },
            CommandType::Pop => {
                stack.pop();
            },
            CommandType::Add => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a + b);
            },
            CommandType::Subtract => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b - a);
            },
            CommandType::Multiply => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a * b);
            },
            CommandType::Divide => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b / a);
            },
            CommandType::Mod => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(b % a);
            },
            CommandType::Not => {
                let x = stack.pop().unwrap();
                stack.push(if x != 0 {
                    0
                } else {
                    1
                });
            }
            CommandType::Greater => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(if b > a {
                    1
                } else {
                    0
                });
            },
            CommandType::Pointer => todo!(),
            CommandType::Switch => todo!(),
            CommandType::Duplicate => {
                let x = stack.pop().unwrap();
                stack.push(x);
                stack.push(x);
            },
            CommandType::Roll => {
                // https://piet.forumotion.com/t7-roll-implementation
                let num_rolls = stack.pop().unwrap();
                let depth = stack.pop().unwrap() as usize;

                let end_ptr = stack.len()-1;
                for _ in 0..num_rolls {
                    for i in 0..depth-1 {
                        stack.swap(end_ptr-i, end_ptr-i-1);
                    }
                }

                // todo: handle negative num_rolls
            },
            CommandType::InNumber => todo!(),
            CommandType::InChar => todo!(),
            CommandType::OutNumber => {
                println!("{}", stack.pop().unwrap());
            },
            CommandType::OutChar => {
                println!("{}", char::from_u32(stack.pop().unwrap() as u32).unwrap());
            },
            CommandType::Branch => {
                command_num = comm.value as usize;
            },
            CommandType::DebugStack => {
                print!("Stack: ");
                for x in stack.iter().rev() {
                    print!("{} ", x);
                }
                println!();
            },
            CommandType::NoOp => {
                // Do nothing
            }
        }
    }

    (stack, dp, cc)
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
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![7, 6, 5, 4, 1, 3, 2]);
    }

    #[test]
    fn test_add() {
        let cmds = read_file("tests/fixtures/add.txt");
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![3]);
    }

    #[test]
    fn test_sub() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("push 4"),
            Command::parse("-")
        ];
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![-1]);
    }

    #[test]
    fn test_multiply() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("push 4"),
            Command::parse("push 5"),
            Command::parse("*")
        ];
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![3, 20]);
    }

    #[test]
    fn test_divide() {
        let cmds = vec![
            Command::parse("push 11"),
            Command::parse("push 2"),
            Command::parse("/")
        ];
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![5]);
    }

    #[test]
    fn test_not() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("not"),
            Command::parse("push 0"),
            Command::parse("not"),
            Command::parse("push 1"),
            Command::parse("not")
        ];
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![0, 1, 0]);
    }

    #[test]
    fn test_greater() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("push 0"),
            Command::parse(">"),
        ];
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![1]);

        let cmds = vec![
            Command::parse("push 1"),
            Command::parse("push 6"),
            Command::parse(">")
        ];
        let (stack, _, _) = run_code(cmds);
        assert_eq!(stack, vec![0]);
    }
}