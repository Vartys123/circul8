mod lib;
use lib::{Env, Program};

fn main() {
    let code = "[-*]/";
    let program = Program::new(code, true);

    let mut env = Env::new(&program, vec![255]);

    println!("Initial state: {:?}", env.cells);
    while env.step() {}
    println!("Final state:   {:?}", env.cells);
}
