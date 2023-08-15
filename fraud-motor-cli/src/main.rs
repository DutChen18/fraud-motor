pub mod dump;
pub mod scan;
pub mod state;
pub mod write;

use clap::Parser;
use rustyline::DefaultEditor;
use state::State;
use std::error::Error;

#[derive(Parser)]
struct Args {
    pid: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut rl = DefaultEditor::new()?;
    let mut state = State::new(args.pid)?;

    loop {
        match rl.readline("(fm) ") {
            Ok(line) => {
                let cmd: Vec<_> = line.split_whitespace().collect();

                match cmd.first() {
                    Some(&"exit") => break,
                    Some(&"dump") => match dump::Args::try_parse_from(&cmd) {
                        Ok(args) => dump::dump(&mut state, args)?,
                        Err(err) => err.print()?,
                    },
                    Some(&"scan") => match scan::Args::try_parse_from(&cmd) {
                        Ok(args) => scan::scan(&mut state, args)?,
                        Err(err) => err.print()?,
                    },
                    Some(&"write") => match write::Args::try_parse_from(&cmd) {
                        Ok(args) => write::write(&mut state, args)?,
                        Err(err) => err.print()?,
                    },
                    Some(&name) => {
                        println!("{}: command not found", name);
                    }
                    _ => {}
                };
            }
            Err(err) => return Err(err.into()),
        };
    }

    Ok(())
}
