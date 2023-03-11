use pretty_assertions::StrComparison;
use colored::Colorize;
use onoi;
use clap::Parser;
use std::{
    fs::{self, File},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};


/// Recursively lists all files under `dir`
fn list_tests_rec(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.append(&mut list_tests_rec(&path)?);
            } else {
                let is_test = match path.extension() {
                    Some(ex) if ex == "ono-test" => true,
                    _ => false,
                };
                if is_test {
                    files.push(path);
                }
            }
        }
    }

    Ok(files)
}

fn run_test(path: &Path) -> Result<(), String> {
    // Disable coloring of error messages
    // otherwise string comparison between errors breaks
    colored::control::set_override(false);

    let file = File::open(path).unwrap(); // unwrap is safe here, this binary found the files itself
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).expect("should be able to read file discovered by binary");
    
    let (code, rest) = match contents.split_once("--ERR--") {
        Some((code, rest)) => (code.trim(), rest),
        _ => panic!("{:?} must contain --ERR-- section", path)
    };

    // TODO: Match out with something
    let (err, _out) = match rest.split_once("--OUT--") {
        Some((err, out)) => (err.trim(), out.trim()),
        _ => panic!("{:?} must contain --OUT-- section", path)
    };

    let result = match onoi::run(code) {
        Ok(()) => Ok(()),
        Err(mut errors) => {
            let filename = path.to_str().unwrap();
            let lines = code.split('\n').collect::<Vec<_>>();
            let error_str = errors.iter_mut().map(|err| {
                let line = lines.get(err.token.position.line).expect("src line refered by error should be in src code");
                err.with_filename(filename);
                err.with_src_line(line);
                format!("{}", err)
            }).collect::<Vec<_>>().join("\n");
            
            if error_str != err {
                let comparison = StrComparison::new(&error_str, err);
                Err(comparison.to_string())
            } else {
                Ok(())
            }
        }
    };

    // undo override of colored output
    colored::control::unset_override();
    result
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the directory containing ".ono-test" files
    #[arg(long, short)]
    dir: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let tests = list_tests_rec(&PathBuf::from(args.dir))?;
    for test in &tests {
        if let Err(error) = run_test(test) {
            println!("{}\t{:?}\n{}", "fail".to_string().red(), test, error);
        } else {
            println!("{}\t{:?}","pass".to_string().green(), test);
        }
    }

    Ok(())
}
