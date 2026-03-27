use std::io::{self, Write};
use std::env;
pub use piet::{Command, CommandType, CodelChooser, DirectionPointer, read_file};

fn main() {
    let args: Vec<String> = env::args().collect();
    let commands = read_file(args.get(1).unwrap_or(&"program.txt".to_string()));

    run_code(commands, false, &mut io::stdout());
}


fn run_code(commands:Vec<Command>, debug: bool, writer: &mut impl Write) -> (Vec<i32>, Vec<String>, DirectionPointer, CodelChooser) {
    let mut stack: Vec<i32> = Vec::new();
    let mut labels: Vec<&str> = Vec::new();
    let mut dp: DirectionPointer = DirectionPointer::Right;
    let mut cc: CodelChooser = CodelChooser::Left;

    let mut command_num: usize = 0;

    while command_num < commands.len() {
        let comm = &commands[command_num];
        if debug {
            println!("{}: {}", command_num, comm.source);
        }
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
                println!("{} - {}", b, a);
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
            CommandType::Pointer => {
                let times = stack.pop().unwrap();
                labels.pop();
                if times > 0 {
                    for _ in 0..times {
                        dp = match dp {
                            DirectionPointer::Right => DirectionPointer::Down,
                            DirectionPointer::Down => DirectionPointer::Left,
                            DirectionPointer::Left => DirectionPointer::Up,
                            DirectionPointer::Up => DirectionPointer::Right,
                        }
                    }
                } else if times < 0 {

                    for _ in 0..times.abs() {
                        dp = match dp {
                            DirectionPointer::Right => DirectionPointer::Up,
                            DirectionPointer::Down => DirectionPointer::Right,
                            DirectionPointer::Left => DirectionPointer::Down,
                            DirectionPointer::Up => DirectionPointer::Left,
                        }
                    }
                }
            },
            CommandType::Switch => {
                let times = stack.pop().unwrap();
                labels.pop();
                if times.abs() % 2 == 1 {
                    cc = match cc {
                        CodelChooser::Left => CodelChooser::Right,
                        CodelChooser::Right => CodelChooser::Left,
                    }
                }
            },
            CommandType::Duplicate => {
                let x = stack.pop().unwrap();
                // Don't pop the label, because we put this right back
                stack.push(x);
                stack.push(x);
                labels.push(&comm.label);
            },
            CommandType::Roll => {
                // https://piet.forumotion.com/t7-roll-implementation
                // https://twoguysarguing.wordpress.com/2010/03/15/piet-roll-command/
                // The ROLL command pops two values off of the stack puts nothing back.
                // The top value from the stack defines how many "turns" the roll executes.
                // A turn will put the top value on the bottom and shift all other values "up" one place toward the top of the stack.
                // The second value defines the "depth" of the roll or how many elements should be included in each turn starting at the top of the stack.

                // As the main Piet page suggests, a ROLL to a depth of X and a single turn will effectively bury the top of the stack down X spots
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
                write!(writer, "{}", stack.pop().unwrap()).unwrap();
                labels.pop();
            },
            CommandType::OutChar => {
                write!(writer, "{}", char::from_u32(stack.pop().unwrap() as u32).unwrap()).unwrap();
                labels.pop();
            },
            CommandType::Branch => {
                let x = stack.pop().unwrap();
                labels.pop();

                if x != 0 {
                    command_num = comm.value as usize;
                }
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
            CommandType::OutLabel => {
                write!(writer, "{}", labels.pop().unwrap()).unwrap();
                stack.pop();
            },
            CommandType::NoOp => {
                // Do nothing
            }
        }
    }

    let ret_labels: Vec<String> = labels.iter().map(|x| x.to_string()).collect();

    (stack, ret_labels, dp, cc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll() {
        let cmds = read_file("tests/fixtures/roll.txt");
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![7, 6, 5, 4, 1, 3, 2]);
        assert_eq!(labels, vec!["a", "b", "c", "d", "g", "e", "f"]);

        // swap top 2 items
        let cmds: Vec<Command> = vec![
            "push 1",
            "push 2",
            "push 3",
            "push 4",
            "push 5",

            "push 2",
            "push 3",
            "roll",
        ].iter().map(|x| Command::parse(x)).collect();

        let (stack, _, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![1, 2, 3, 5, 4]); // Rightmost is the top of the stack


        let cmds: Vec<Command> = vec![
            "push 1",
            "push 2",
            "push 3",
            "push 4",
            "push 5",
            "push 3",
            "push 1",
            "roll",
        ].iter().map(|x| Command::parse(x)).collect();

        let (stack, _, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![1, 2, 5, 3, 4]);

        let cmds: Vec<Command> = vec![
            "push 7",
            "push 6",
            "push 5",
            "push 4",
            "push 3",
            "push 2",
            "push 1",

            "push 3",
            "push 1",
            "roll",
        ].iter().map(|x| Command::parse(x)).collect();

        let (stack, _, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![7,6,5,4,1,3,2]);

        // example of getting the third entry to the top
        let cmds: Vec<Command> = vec![
            "push 5",
            "push 4",
            "push 3",
            "push 2",
            "push 1",

            "push 3",
            "push 2",
            "roll",
        ].iter().map(|x| Command::parse(x)).collect();

        let (stack, _, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![5,4,2,1,3]);
    }

    #[test]
    fn test_add() {
        let cmds = read_file("tests/fixtures/add.txt");
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
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
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
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
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
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
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
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
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
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
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![1]);
        assert_eq!(labels, vec!["answer"]);

        let cmds = vec![
            Command::parse("push 1 a"),
            Command::parse("push 6 b"),
            Command::parse("> answer")
        ];
        let (stack, labels, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![0]);
        assert_eq!(labels, vec!["answer"]);
    }

    #[test]
    fn test_branch() {
        let cmds = vec![
            Command::parse("push -3"),
            Command::parse("push 1"),
            Command::parse("add"),
            Command::parse("duplicate"),
            Command::parse("debug_stack"),
            Command::parse("branch 1"),
        ];
        let (stack, _, _, _) = run_code(cmds, true, &mut vec![]);
        assert_eq!(stack, vec![0]);
    }

    #[test]
    fn test_pointer() {
        let cmds = vec![
            Command::parse("push 3"),
            Command::parse("pointer"),
        ];
        let (stack, labels, dp, _) = run_code(cmds, true, &mut vec![]);
        assert!(stack.is_empty());
        assert!(labels.is_empty());
        assert_eq!(dp, DirectionPointer::Up);


        let cmds = vec![
            Command::parse("push -3"),
            Command::parse("pointer"),
        ];
        let (stack, labels, dp, _) = run_code(cmds, true, &mut vec![]);
        assert!(stack.is_empty());
        assert!(labels.is_empty());
        assert_eq!(dp, DirectionPointer::Down);
    }

    #[test]
    fn test_switch() {
        let cmds = vec![
            Command::parse("push 1"),
            Command::parse("switch"),
        ];
        let (stack, labels, _, cc) = run_code(cmds, true, &mut vec![]);
        assert!(stack.is_empty());
        assert!(labels.is_empty());
        assert_eq!(cc, CodelChooser::Right);


        let cmds = vec![
            Command::parse("push 2"),
            Command::parse("switch"),
        ];
        let (stack, labels, _, cc) = run_code(cmds, true, &mut vec![]);
        assert!(stack.is_empty());
        assert!(labels.is_empty());
        assert_eq!(cc, CodelChooser::Left);

        let cmds = vec![
            Command::parse("push -1"),
            Command::parse("switch"),
        ];
        let (stack, labels, _, cc) = run_code(cmds, true, &mut vec![]);
        assert!(stack.is_empty());
        assert!(labels.is_empty());
        assert_eq!(cc, CodelChooser::Right);
    }

    #[test]
    fn test_count_down() {
        let cmds = read_file("tests/fixtures/count_down.txt");
        let mut output: Vec<u8> = Vec::new();
        let (stack, _, _, _) = run_code(cmds, true, &mut output);

        // Loop prints 100, 95, ..., -95, -100 then subtracts one more step leaving -105
        assert_eq!(stack, vec![-105]);

        let expected: String = ((-100..=100).rev().step_by(5))
            .map(|n| format!("{}\n", n))
            .collect();
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_count_down_nested() {
        let cmds = read_file("tests/fixtures/count_down_nested.txt");
        let mut output: Vec<u8> = Vec::new();
        let (stack, _, _, _) = run_code(cmds, true, &mut output);

        assert_eq!(stack, vec![-105]);

        let mut expected = String::new();
        let mut i = 100i64;
        loop {
            expected.push_str(&format!("{}\n", i));
            let mut j = -200i64;
            loop {
                expected.push_str(&format!("{}\n", j));
                j += 3;
                if j > 50 { break; }
            }
            i -= 5;
            if !(i > -101) { break; }
        }
        assert_eq!(String::from_utf8(output).unwrap(), expected);
    }

    #[test]
    fn test_mandelbrot_complex() {
        let program = read_file("tests/fixtures/mandelbrot_complex.txt");

        let mut test_1_1 =  vec![
            Command::parse("push 1 a"),
            Command::parse("push 1 b"),
        ];
        test_1_1.extend(program.clone());
        let (stack, _, _, _) = run_code(test_1_1, true, &mut vec![]);
        assert_eq!(stack, vec![0]);

        let mut test_5_5 =  vec![
            Command::parse("push 5 a"),
            Command::parse("push 5 b"),
        ];
        test_5_5.extend(program.clone());
        let (stack, _, _, _) = run_code(test_5_5, true, &mut vec![]);
        assert_eq!(stack, vec![0]);

        let mut test_20_30 =  vec![
            Command::parse("push 20 a"),
            Command::parse("push 30 b"),
        ];
        test_20_30.extend(program.clone());
        let (stack, _, _, _) = run_code(test_20_30, true, &mut vec![]);
        assert_eq!(stack, vec![11]);

        let mut test_50_70 =  vec![
            Command::parse("push -50 a"),
            Command::parse("push -70 b"),
        ];
        test_50_70.extend(program.clone());
        let (stack, _, _, _) = run_code(test_50_70, true, &mut vec![]);
        assert_eq!(stack, vec![325]); //Ruby will give very different answers, due to stupid integer division rules
    }

    #[test]
    fn test_mandelbrot_complex_print() {
        let program = read_file("tests/fixtures/mandelbrot_complex_print.txt");

        let mut test =  vec![
            Command::parse("push 5 a"),
            Command::parse("push 5 b"),
        ];
        test.extend(program.clone());
        let mut output: Vec<u8> = Vec::new();
        let (stack, _, _, _) = run_code(test, true, &mut output);
        assert_eq!(String::from_utf8(output).unwrap(), " ");
        assert_eq!(stack, vec![]);


        let mut test =  vec![
            Command::parse("push -50 a"),
            Command::parse("push -70 b"),
        ];
        test.extend(program.clone());
        let mut output: Vec<u8> = Vec::new();
        let (stack, _, _, _) = run_code(test, true, &mut output);
        assert_eq!(String::from_utf8(output).unwrap(), "*");
        assert_eq!(stack, vec![]);

    }
}
