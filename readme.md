The following describes a stack-based language. Each line of the program contains a single instruction, optionally followed by a space then a number, which would the value passed to the command. '#' begins a comment; everything from there to the end of the line is ignored.

These are the valid commands:
- push x: Pushes x on to the stack.
- pop: Pops the top value off the stack and discards it.
- add: Pops the top two values off the stack, adds them, and pushes the result back on the stack.
- subtract: Pops the top two values off the stack, calculates the second top value minus the top value, and pushes the result back on the stack.
- multiply: Pops the top two values off the stack, multiplies them, and pushes the result back on the stack.
- divide: Pops the top two values off the stack, calculates the integer division of the second top value by the top value, and pushes the result back on the stack. If a divide by zero occurs, it is handled as an implementation-dependent error, though simply ignoring the command is recommended.
- mod: Pops the top two values off the stack, calculates the second top value modulo the top value, and pushes the result back on the stack. The result has the same sign as the divisor (the top value). If the top value is zero, this is a divide by zero error, which is handled as an implementation-dependent error, though simply ignoring the command is recommended. (See note below.)
- not: Replaces the top value of the stack with 0 if it is non-zero, and 1 if it is zero.
- greater: Pops the top two values off the stack, and pushes 1 on to the stack if the second top value is greater than the top value, and pushes 0 if it is not greater.
- duplicate: Pushes a copy of the top value on the stack on to the stack.
- roll: Pops the top two values off the stack and "rolls" the remaining stack entries to a depth equal to the second value popped, by a number of rolls equal to the first value popped. A single roll to depth n is defined as burying the top value on the stack n deep and bringing all values above it up by 1 place. A negative number of rolls rolls in the opposite direction. A negative depth is an error and the command is ignored. If a roll is greater than an implementation-dependent maximum stack depth, it is handled as an implementation-dependent error, though simply ignoring the command is recommended.
out_number: Pops the top value off the stack and prints it to STDOUT as a number.
- out_char: Pops the top value off the stack and prints it to STDOUT as the equivalent ascii character.
- branch x: pops the top value off the stack and jumps to the line number indicated by x if the value was non-zero. Line numbers are zero-indexed.

Examples

Here is a simple program:
```
push 1
push 2
add
```
This ends with the value `3` on the stack.

Here is an example of the `roll` command:
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
