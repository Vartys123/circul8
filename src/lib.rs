use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::sync::Arc;

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

#[pyclass(from_py_object)]
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
                        b'+' | b'-' => {
                            let mut r = n;
                            while r > 255 {
                                instrs.push(if c == b'+' {
                                    Instruction::Increment(255)
                                } else {
                                    Instruction::Decrement(255)
                                });
                            }
                            instrs.push(if c == b'+' {
                                Instruction::Increment(r as u8)
                            } else {
                                Instruction::Decrement(r as u8)
                            });
                        }
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
                _ => i += 1,
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

#[pymethods]
impl Program {
    #[new]
    #[pyo3(signature = (code, fold=true))]
    pub fn py_new(code: &str, fold: bool) -> PyResult<Self> {
        Ok(Self::new(code, fold))
    }
    pub fn get_instructions_asm(&self) -> Vec<String> {
        self.instructions
            .iter()
            .map(|i| format!("{:?}", i))
            .collect()
    }
    pub fn __str__(&self) -> String {
        delex(&self.instructions)
    }
}

#[pyclass]
#[derive(Debug)]
pub struct Env {
    #[pyo3(get, set)]
    pub cells: Vec<u8>,
    #[pyo3(get, set)]
    pub cell_ptr: usize,
    pub program: Arc<Program>,
    #[pyo3(get, set)]
    pub instr_ptr: usize,
}

#[pymethods]
impl Env {
    #[new]
    pub fn py_new(py: Python<'_>, program: Py<Program>, input: Vec<u8>) -> Self {
        let p_ref = program.bind(py).borrow();

        let mut cells = input;
        if cells.is_empty() {
            cells.push(0);
        }

        Self {
            cells,
            cell_ptr: 0,
            program: Arc::new(p_ref.clone()),
            instr_ptr: 0,
        }
    }
    pub fn step(&mut self) -> bool {
        let Some(&instr) = self.program.instructions.get(self.instr_ptr) else {
            return false;
        };
        match instr {
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
    pub fn run(&mut self, max_steps: usize) -> usize {
        let mut count = 0;
        while count < max_steps && self.step() {
            count += 1;
        }
        count
    }
    #[getter]
    pub fn get_cells_as_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.cells)
    }
}

impl Env {
    pub fn new(program: Arc<Program>, input: Vec<u8>) -> Self {
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
}

#[pymodule]
fn circul8(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Program>()?;
    m.add_class::<Env>()?;
    Ok(())
}

pub fn delex(instructions: &[Instruction]) -> String {
    let len: usize = instructions
        .iter()
        .map(|i| match i {
            Instruction::Right(n) | Instruction::Left(n) => *n,
            Instruction::Increment(n) | Instruction::Decrement(n) => *n as usize,
            _ => 1,
        })
        .sum();
    let mut s = String::with_capacity(len);
    for i in instructions {
        match i {
            Instruction::Right(n) => push_chars(&mut s, '>', *n),
            Instruction::Left(n) => push_chars(&mut s, '<', *n),
            Instruction::Increment(n) => push_chars(&mut s, '+', *n as usize),
            Instruction::Decrement(n) => push_chars(&mut s, '-', *n as usize),
            Instruction::LoopStart(..) => s.push('['),
            Instruction::LoopEnd(..) => s.push(']'),
            Instruction::Push => s.push('*'),
            Instruction::Pop => s.push('/'),
        }
    }
    s
}

fn push_chars(s: &mut String, c: char, n: usize) {
    for _ in 0..n {
        s.push(c);
    }
}
