use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut buffer: Vec<u8> = Vec::new();
    File::open("challenge.bin")
        .expect("Could not open file")
        .read_to_end(&mut buffer)
        .unwrap();

    let ints: &Vec<u16> = &buffer[..]
        .chunks(2)
        .map(|x| u16::from_le_bytes(*<&[u8; 2]>::try_from(x).unwrap()))
        .collect();

    run_program(&mut ints.clone());
}

fn write(
    memory: &mut [u16; u16::MAX as usize],
    registers: &mut [u16; 8],
    position: u16,
    data: u16,
) {
    match position {
        0..=32767 => memory[position as usize] = data,
        32767..=32775 => registers[(position - 32767) as usize] = data,
        _ => unreachable!(),
    };
}

fn read(memory: &mut [u16; u16::MAX as usize], registers: &mut [u16; 8], position: u16) -> u16 {
    match position {
        0..=32767 => memory[position as usize],
        32768..=32775 => registers[(position - 32767) as usize],
        _ => unreachable!(),
    }
}

fn run_program(ints: &mut Vec<u16>) {
    let mut cursor = 0;

    let mut registers: [u16; 8] = [0; 8];
    let mut stack: Vec<u16> = Vec::new();
    let mut memory: [u16; u16::MAX as usize] = [0; u16::MAX as usize];
    let mut stdin_chars: Vec<u16> = Vec::new();

    for i in 0..ints.len() {
        memory[i] = ints[i];
    }

    loop {
        let a = memory[cursor + 1];
        let b = memory[cursor + 2];
        let c = memory[cursor + 3];

        match ints[cursor] {
            0 => {
                // stop execution and terminate the program
                println!("Exiting at {}", cursor);
                return;
            }
            1 => {
                // set register <a> to the value of <b>
                registers[a as usize] = b;
                cursor += 2;
            }
            2 => {
                // push <a> onto the stack
                stack.push(a);
                cursor += 1;
            }
            3 => {
                // remove the top element from the stack and write it into <a>;
                // empty stack = error
                write(&mut memory, &mut registers, a, stack.pop().unwrap());
                cursor += 1;
            }
            4 => {
                // set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                if b == c {
                    write(&mut memory, &mut registers, (cursor + 1) as u16, 1);
                } else {
                    write(&mut memory, &mut registers, (cursor + 1) as u16, 0);
                }
                cursor += 3;
            }
            5 => {
                // set <a> to 1 if <b> is greater than <c>; set it to 0
                // otherwise
                if b > c {
                    write(&mut memory, &mut registers, (cursor + 1) as u16, 1);
                } else {
                    write(&mut memory, &mut registers, (cursor + 1) as u16, 0);
                }
                cursor += 3;
            }
            6 => {
                // jump to <a>
                cursor = (a - 1) as usize;
            }
            7 => {
                // if <a> is nonzero, jump to <b>
                if a != 0 {
                    cursor = (b - 1) as usize;
                } else {
                    cursor += 2;
                }
            }
            8 => {
                // if <a> is zero, jump to <b>
                if a == 0 {
                    cursor = (b - 1) as usize;
                } else {
                    cursor += 2;
                }
            }
            9 => {
                // assign into <a> the sum of <b> and <c> (modulo 32768)
                write(&mut memory, &mut registers, a, (b + c) % 32768);
                cursor += 3;
            }
            10 => {
                // assign into <a> the product of <b> and <c> (modulo 32768)
                write(&mut memory, &mut registers, a, (b * c) % 32768);
                cursor += 3;
            }
            11 => {
                // store into <a> the remainder of <b> divided by <c>
                write(&mut memory, &mut registers, a, (b % c) % 32768);
                cursor += 3;
            }
            12 => {
                // stores into <a> the bitwise and of <b> and <c>
                write(&mut memory, &mut registers, a, b & c);
                cursor += 3;
            }
            13 => {
                // stores into <a> the bitwise or of <b> and <c>
                write(&mut memory, &mut registers, a, b | c);
                cursor += 3;
            }
            14 => {
                // stores 15-bit bitwise inverse of <b> in <a>
                write(&mut memory, &mut registers, a, !b);
                cursor += 2;
            }
            15 => {
                // read memory at address <b> and write it to <a>
                let b_data = read(&mut memory, &mut registers, b);
                write(&mut memory, &mut registers, a, b_data);
                cursor += 2;
            }
            16 => {
                // write the value from <b> into memory at address <a>
                write(&mut memory, &mut registers, a, b);
                cursor += 2;
            }
            17 => {
                // write the address of the next instruction to the stack and
                // jump to <a>
                stack.push(b);
                cursor = (a - 1) as usize;
            }
            18 => {
                // remove the top element from the stack and jump to it; empty
                // stack = halt
                if stack.len() == 0 {
                    println!("Exiting from empty stack at {}", cursor);
                    return;
                }

                cursor = (stack.pop().unwrap() - 1) as usize;
            }
            19 => {
                // write the character represented by ascii code <a> to the
                // terminal
                print!("{}", a as u8 as char);
                cursor += 1;
            }
            20 => {
                // read a character from the terminal and write its ascii code
                // to <a>; it can be assumed that once input starts, it will
                // continue until a newline is encountered; this means that you
                // can safely read whole lines from the keyboard and trust that
                // they will be fully read

                // If there is nothing else to read in, ask for more input
                if stdin_chars.len() == 0 {
                    let mut stdin = String::new();

                    println!("Enter input: ");
                    std::io::stdin().read_line(&mut stdin).unwrap().to_string();

                    stdin_chars = stdin.chars().map(|x| x as u16).collect();
                }
                write(&mut memory, &mut registers, a, stdin_chars.remove(0));
            }
            21 => {
                // noop
            }
            _ => (),
        }

        cursor += 1;
        // println!("{}", cursor);
        if cursor % 10 == 0 {}
    }
}
