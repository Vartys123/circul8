#[derive(Debug, Clone, Copy, PartialEq)]
enum Instruction {
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
struct Program {
    instructions: Vec<Instruction>,
    jumps: Vec<usize>,
}

impl Program {
    fn new(instructions: Vec<Instruction>) -> Self {
        let mut jumps = vec![0; instructions.len()];
        let mut stack = Vec::new();

        for (i, instr) in instructions.iter().enumerate() {
            match instr {
                Instruction::LoopStart => stack.push(i),
                Instruction::LoopEnd => {
                    if let Some(start_i) = stack.pop() {
                        jumps[start_i] = i;
                        jumps[i] = start_i;
                    }
                }
                _ => {}
            }
        }

        Self {
            instructions,
            jumps,
        }
    }
}

#[derive(Debug)]
struct Env<'a> {
    cells: Vec<u8>,
    cell_ptr: usize,
    program: &'a Program,
    instr_ptr: usize,
}

impl<'a> Env<'a> {
    fn new(program: &'a Program, input: Vec<u8>) -> Self {
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

    fn parse(&mut self, max_steps: usize) -> usize {
        let mut steps = 0;
        let prog_len = self.program.instructions.len();

        while self.instr_ptr < prog_len && steps < max_steps {
            match &self.program.instructions[self.instr_ptr] {
                Instruction::Right => {
                    self.cell_ptr = (self.cell_ptr + 1) % self.cells.len();
                }
                Instruction::Left => {
                    if self.cell_ptr == 0 {
                        self.cell_ptr = self.cells.len() - 1
                    } else {
                        self.cell_ptr -= 1
                    }
                }
                Instruction::Increment => {
                    self.cells[self.cell_ptr] = self.cells[self.cell_ptr].wrapping_add(1);
                }
                Instruction::Decrement => {
                    self.cells[self.cell_ptr] = self.cells[self.cell_ptr].wrapping_sub(1);
                }
                Instruction::LoopStart => {
                    if self.cells[self.cell_ptr] == 0 {
                        let target = self.program.jumps[self.instr_ptr];
                        if target > self.instr_ptr {
                            self.instr_ptr = target;
                        }
                    }
                }
                Instruction::LoopEnd => {
                    if self.cells[self.cell_ptr] != 0 {
                        let target = self.program.jumps[self.instr_ptr];
                        if target < self.instr_ptr {
                            self.instr_ptr = target;
                        }
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
            steps += 1;
        }
        steps
    }
}

fn lex(text: &str) -> Vec<Instruction> {
    text.chars()
        .filter_map(|c| match c {
            '>' => Some(Instruction::Right),
            '<' => Some(Instruction::Left),
            '+' => Some(Instruction::Increment),
            '-' => Some(Instruction::Decrement),
            '[' => Some(Instruction::LoopStart),
            ']' => Some(Instruction::LoopEnd),
            '*' => Some(Instruction::Push),
            '/' => Some(Instruction::Pop),
            _ => None,
        })
        .collect()
}

fn delex(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(|i| match i {
            Instruction::Right => '>',
            Instruction::Left => '<',
            Instruction::Increment => '+',
            Instruction::Decrement => '-',
            Instruction::LoopStart => '[',
            Instruction::LoopEnd => ']',
            Instruction::Push => '*',
            Instruction::Pop => '/',
        })
        .collect()
}

fn main() {
    let code = "[-*]/";
    let instructions = lex(code);
    let program = Program::new(instructions);

    let mut env = Env::new(&program, vec![255]);

    println!("Initial state: {:?}", env.cells);
    let steps = env.parse(1024);
    println!("Final state:   {:?}", env.cells);
    println!("Executed:      {} steps", steps);
}
