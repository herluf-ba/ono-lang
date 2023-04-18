use anyhow::Result;
use clap::Parser;
use common::error::SyntaxError;
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

fn main() -> Result<()> {
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

    let tokens = syntax::Scanner::new(&code)
        .into_iter()
        .collect::<Result<Vec<_>, _>>();
    match tokens {
        Err(err) => {
            let filename = args.file.file_name().map(|f| f.to_str());
            if let Some(Some(filename)) = filename {
                common::error::report_errors::<SyntaxError>(filename, &code, vec![err])?;
            }
        }
        Ok(tokens) => println!("{:?}", tokens),
    };

    Ok(())
}
