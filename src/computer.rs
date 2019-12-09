use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::convert::TryInto;
use std::collections::HashMap;

//const COMMAND_MAX_LENGTH: usize = 4;

#[derive(Debug)]
pub struct Computer {
    memory: Memory,
}

impl Computer {
    pub fn new(memory: Vec<i64>) -> Self {
        Self { memory: memory.into() }
    }

    pub fn run(&mut self, input: Receiver<i64>, output: Sender<i64>) {
        let mut ptr = 0;
        let mut relative_offset = 0;
        loop {
            //println!("Pointer: {}", ptr);
            //println!("Memory: {:?}", self.memory);
            let command = Command::parse(&self.memory, ptr);
            println!("Running instruction {:?}", command);
            let mut jumped = false;
            match &command {
                Command::Add(a, b, c) => {
                    let first_val = a.unwrap(&self.memory, relative_offset);
                    let second_val = b.unwrap(&self.memory, relative_offset);
                    let sum = first_val + second_val;
                    match c {
                        Pointer::Position(p) => { *self.memory.get_mut(*p) = sum }
                        Pointer::Relative(p) => { *self.memory.get_mut((*p + relative_offset).try_into().unwrap()) = sum }
                        Pointer::Value(_) => { panic!("Can't store value in a literal value") }
                    }
                }
                Command::Multiply(a, b, c) => {
                    let first_val = a.unwrap(&self.memory, relative_offset);
                    let second_val = b.unwrap(&self.memory, relative_offset);
                    let product = first_val * second_val;
                    match c {
                        Pointer::Position(p) => { *self.memory.get_mut(*p) = product }
                        Pointer::Relative(p) => { *self.memory.get_mut((*p + relative_offset).try_into().unwrap()) = product }
                        Pointer::Value(_) => { panic!("Can't store value in a literal value") }
                    }
                }
                Command::Output(a) => {
                    output.send(a.unwrap(&self.memory, relative_offset)).unwrap();
                }
                Command::Input(a) => {
                    match a {
                        Pointer::Value(_) => { panic!("Can't store value in a literal value") }
                        Pointer::Position(p) => { *self.memory.get_mut(*p) = input.recv().unwrap() }
                        Pointer::Relative(p) => { *self.memory.get_mut((relative_offset + *p).try_into().unwrap()) = input.recv().unwrap() }
                    }
                }
                Command::JumpTrue(a, b) => {
                    if a.unwrap(&self.memory, relative_offset) != 0 {
                        ptr = b.unwrap(&self.memory, relative_offset) as usize;
                        jumped = true;
                    }
                }
                Command::JumpFalse(a, b) => {
                    if a.unwrap(&self.memory, relative_offset) == 0 {
                        ptr = b.unwrap(&self.memory, relative_offset) as usize;
                        jumped = true;
                    }
                }
                Command::LessThan(a, b, c) => {
                    match c {
                        Pointer::Position(p) => {
                            *self.memory.get_mut(*p) =
                                if a.unwrap(&self.memory, relative_offset) < b.unwrap(&self.memory, relative_offset) { 1 } else { 0 }
                        }
                        Pointer::Relative(p) => {
                            *self.memory.get_mut((relative_offset + *p).try_into().unwrap()) =
                                if a.unwrap(&self.memory, relative_offset) < b.unwrap(&self.memory, relative_offset) { 1 } else { 0 }
                        }
                        Pointer::Value(_) => { panic!("Can't store value in a literal value") }
                    }
                }
                Command::Equals(a, b, c) => {
                    match c {
                        Pointer::Position(p) => {
                            *self.memory.get_mut(*p) =
                                if a.unwrap(&self.memory, relative_offset) == b.unwrap(&self.memory, relative_offset) { 1 } else { 0 }
                        }
                        Pointer::Relative(p) => {
                            *self.memory.get_mut((relative_offset + *p).try_into().unwrap()) =
                                if a.unwrap(&self.memory, relative_offset) == b.unwrap(&self.memory, relative_offset) { 1 } else { 0 }
                        }
                        Pointer::Value(_) => { panic!("Can't store value in a literal value") }
                    }
                }
                Command::ChangeOffset(a) => {
                    let val = a.unwrap(&self.memory, relative_offset) as isize;
                    relative_offset += val;
                }
                Command::Return => { break }
            }

            if !jumped {
                ptr += command.len();
            }
        }
    }

    pub fn run_blocking(mut self, input: &[i64]) -> Vec<i64> {
        let (input_transmitter, input_receiver) = mpsc::channel();
        let (output_transmitter, output_receiver) = mpsc::channel();
    
        for data in input {
            input_transmitter.send(*data).unwrap();
        }
        
        self.run(input_receiver, output_transmitter);
        
        let mut out = Vec::new();
    
        loop {
            match output_receiver.recv() {
                Ok(d) => out.push(d),
                Err(_) => break,
            }
        }
        
        out
    }
}

#[derive(Debug)]
struct Memory {
    data: HashMap<usize, i64>,
}

impl Memory {
    fn get(&self, index: usize) -> &i64 {
        match self.data.get(&index) {
            Some(t) => {
                t
            }
            None => {
                &0
            }
        }
    }

    fn get_mut(&mut self, index: usize) -> &mut i64 {
        if !self.data.contains_key(&index) {
            self.data.insert(index, 0);
        }
        self.data.get_mut(&index).unwrap()
    }
}

impl From<Vec<i64>> for Memory {
    fn from(p: Vec<i64>) -> Self {
        let mut out = Self { data: HashMap::new() };
        for i in 0..p.len() {
            out.data.insert(i, p[i]);
        }
        out
    }
}

#[derive(Debug, Clone)]
enum Pointer {
    Position(usize),
    Value(i64),
    Relative(isize),
}

impl Pointer {
    fn parse(num_type: i64, addr: impl FnOnce() -> i64) -> impl FnOnce() -> Self {
        move || -> Self {
            match num_type {
                0 => {
                    Pointer::Position(addr() as usize)
                }
                1 => {
                    Pointer::Value(addr())
                }
                2 => {
                    Pointer::Relative(addr() as isize)
                }
                a => {
                    panic!("{}", a);
                }
            }
        }
    }
    
    fn unwrap(&self, program: &Memory, relative_offset: isize) -> i64 {
        match self {
            Pointer::Position(p) => { *program.get(*p) }
            Pointer::Value(v) => { *v }
            Pointer::Relative(p) => { *program.get((relative_offset + *p).try_into().unwrap()) }
        }
    }
}

#[derive(Debug)]
enum Command {
    Add(Pointer, Pointer, Pointer),
    Multiply(Pointer, Pointer, Pointer),
    Output(Pointer),
    Input(Pointer),
    JumpTrue(Pointer, Pointer),
    JumpFalse(Pointer, Pointer),
    LessThan(Pointer, Pointer, Pointer),
    Equals(Pointer, Pointer, Pointer),
    ChangeOffset(Pointer),
    Return,
}

impl Command {
    fn parse(parts: &Memory, index: usize) -> Self {
        let first_ptr = Pointer::parse(parts.get(index) % 1000 / 100, || -> i64 { *parts.get(index + 1) });
        let second_ptr = Pointer::parse(parts.get(index) % 10000 / 1000, || -> i64 { *parts.get(index + 2) });
        let third_ptr = Pointer::parse(parts.get(index) % 100000 / 10000, || -> i64 { *parts.get(index + 3) });
        match parts.get(index) % 100 {
            1 => {
                Command::Add(first_ptr(), second_ptr(), third_ptr())
            }
            2 => {
                Command::Multiply(first_ptr(), second_ptr(), third_ptr())
            }
            3 => {
                Command::Input(first_ptr())
            }
            4 => {
                Command::Output(first_ptr())
            }
            5 => {
                Command::JumpTrue(first_ptr(), second_ptr())
            }
            6 => {
                Command::JumpFalse(first_ptr(), second_ptr())
            }
            7 => {
                Command::LessThan(first_ptr(), second_ptr(), third_ptr())
            }
            8 => {
                Command::Equals(first_ptr(), second_ptr(), third_ptr())
            }
            9 => {
                Command::ChangeOffset(first_ptr())
            }
            99 => {
                Command::Return
            }
            a => { panic!("invalid opcode: {}", a) }
        }
    }
    
    fn len(&self) -> usize {
        match self {
            Command::Add(_, _, _) => { 4 }
            Command::Multiply(_, _, _) => { 4 }
            Command::Output(_) => { 2 }
            Command::Input(_) => { 2 }
            Command::Return => { 1 }
            Command::JumpTrue(_, _) => { 3 }
            Command::JumpFalse(_, _) => { 3 }
            Command::LessThan(_, _, _) => { 4 }
            Command::Equals(_, _, _) => { 4 }
            Command::ChangeOffset(_) => { 2 }
        }
    }
}