use clap::Parser;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

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

    match onoi::run(&code) {
        Err(mut errors) => {
            let lines = code.split("\n").collect::<Vec<_>>();
            for error in errors.iter_mut() {
                let line = lines
                    .get(error.token.position.line)
                    .expect("error line should refer to a line in src code");
                error.with_src_line(line);
                error.with_filename(args.file.to_str().unwrap());
                eprintln!("{}\n", error);
            }

            println!(
                "program exited with {} {}",
                errors.len(),
                if errors.len() > 1 { "errors" } else { "error" }
            );
        }
        Ok(()) => {}
    };
}
