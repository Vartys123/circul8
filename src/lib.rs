#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Right(usize),
    Left(usize),
    Increment(u8),
    Decrement(u8),
    LoopStart(usize),
    LoopEnd(usize),
    Push,
    Pop,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

impl Program {
    pub fn new(text: &str, fold: bool) -> Self {
        let mut instrs = Vec::new();
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            match c {
                b'>' | b'<' | b'+' | b'-' => {
                    let start = i;
                    if fold == true {
                        while i < bytes.len() && bytes[i] == c {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                    let n = i - start;
                    match c {
                        b'>' => instrs.push(Instruction::Right(n)),
                        b'<' => instrs.push(Instruction::Left(n)),
                        b'+' => instrs.push(Instruction::Increment(n as u8)),
                        b'-' => instrs.push(Instruction::Decrement(n as u8)),
                        _ => unreachable!(),
                    }
                }
                b'[' => {
                    instrs.push(Instruction::LoopStart(0));
                    i += 1;
                }
                b']' => {
                    instrs.push(Instruction::LoopEnd(0));
                    i += 1;
                }
                b'*' => {
                    instrs.push(Instruction::Push);
                    i += 1;
                }
                b'/' => {
                    instrs.push(Instruction::Pop);
                    i += 1;
                }
                _ => {}
            }
        }
        let mut stack = Vec::new();
        for idx in 0..instrs.len() {
            if let Instruction::LoopStart(_) = instrs[idx] {
                stack.push(idx);
            } else if let Instruction::LoopEnd(_) = instrs[idx] {
                if let Some(start_idx) = stack.pop() {
                    instrs[start_idx] = Instruction::LoopStart(idx);
                    instrs[idx] = Instruction::LoopEnd(start_idx);
                }
            }
        }
        Self {
            instructions: instrs,
        }
    }
}

#[derive(Debug)]
pub struct Env<'a> {
    pub cells: Vec<u8>,
    pub cell_ptr: usize,
    pub program: &'a Program,
    pub instr_ptr: usize,
}

impl<'a> Env<'a> {
    pub fn new(program: &'a Program, input: Vec<u8>) -> Self {
        let mut cells = input;
        if cells.is_empty() {
            cells.push(0);
        }
        Self {
            cells,
            cell_ptr: 0,
            program,
            instr_ptr: 0,
        }
    }

    pub fn step(&mut self) -> bool {
        let instr = match self.program.instructions.get(self.instr_ptr) {
            Some(i) => i,
            None => return false,
        };
        match *instr {
            Instruction::Right(n) => {
                self.cell_ptr = (self.cell_ptr + n) % self.cells.len();
            }
            Instruction::Left(n) => {
                let len = self.cells.len();
                self.cell_ptr = (self.cell_ptr + len - (n % len)) % len;
            }
            Instruction::Increment(n) => {
                self.cells[self.cell_ptr] = self.cells[self.cell_ptr].wrapping_add(n);
            }
            Instruction::Decrement(n) => {
                self.cells[self.cell_ptr] = self.cells[self.cell_ptr].wrapping_sub(n);
            }
            Instruction::LoopStart(target) => {
                if self.cells[self.cell_ptr] == 0 {
                    self.instr_ptr = target;
                }
            }
            Instruction::LoopEnd(target) => {
                if self.cells[self.cell_ptr] != 0 {
                    self.instr_ptr = target;
                }
            }
            Instruction::Push => {
                self.cells
                    .insert(self.cell_ptr + 1, self.cells[self.cell_ptr]);
            }
            Instruction::Pop => {
                if self.cells.len() > 1 {
                    let target = (self.cell_ptr + 1) % self.cells.len();
                    self.cells.remove(target);
                    self.cell_ptr %= self.cells.len();
                }
            }
        }
        self.instr_ptr += 1;
        true
    }
}

pub fn delex(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(|i| match i {
            Instruction::Right(n) => ">".repeat(*n),
            Instruction::Left(n) => "<".repeat(*n),
            Instruction::Increment(n) => "+".repeat(*n as usize),
            Instruction::Decrement(n) => "-".repeat(*n as usize),
            Instruction::LoopStart(..) => "[".to_string(),
            Instruction::LoopEnd(..) => "]".to_string(),
            Instruction::Push => "*".to_string(),
            Instruction::Pop => "/".to_string(),
        })
        .collect()
}
