use clap::Parser;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
mod compiler;
mod vm;

use compiler::Scanner;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The ono source file to run
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let file = match File::open(&args.file) {
        Ok(f) => f,
        Err(_) => panic!("Could not find {:?}", args.file),
    };

    let mut reader = BufReader::new(file);
    let mut code = String::new();
    if reader.read_to_string(&mut code).is_err() {
        panic!("could not read {:?}", args.file);
    }

    let tokens = Scanner::new(&code).into_iter().collect::<Result<Vec<_>, _>>();
    match tokens {
        Err(why) => println!("{}", why),
        Ok(tokens) => {
            for token in tokens {
                println!("{}", token)
            }
        }
    }
}
// fn main() {
//     let mut test_chunk = Chunk::new();
//     test_chunk.push_constant(1.2, 123);
//     test_chunk.push_constant(3.4, 123);
//     test_chunk.push(OpCode::ADD, 123);
//     test_chunk.push_constant(5.6, 123);
//     test_chunk.push(OpCode::DIVIDE, 123);
//     test_chunk.push(OpCode::NEGATE, 123);
//     test_chunk.push(OpCode::RETURN, 123);
//
//     let mut vm = VirtualMachine::new(test_chunk);
//     println!("{:?}", vm.run());
// }
