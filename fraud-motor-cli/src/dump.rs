use crate::state::State;
use clap::{Parser, Subcommand};
use fraud_motor_core::dump::ProcessDump;
use std::error::Error;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New { name: String },
    Drop { name: String },
    Info { name: Option<String> },
}

pub fn dump(state: &mut State, args: Args) -> Result<(), Box<dyn Error>> {
    match args.command {
        Commands::New { name } => {
            let dump = ProcessDump::new(&state.memory, &state.proc, |region| {
                region.permissions().write()
            })?;

            state.dumps.insert(name, dump);
        }
        Commands::Drop { name } => {
            state.dumps.remove(&name);
        }
        Commands::Info { name } => {
            if let Some(name) = name {
                if let Some(dump) = state.dumps.get(&name) {
                    for (region, data) in dump.regions() {
                        let perms = region.permissions();

                        print!(
                            "{} {:016x}-{:016x} {}{}{}",
                            if data.is_ok() { "ok " } else { "err" },
                            region.start(),
                            region.end(),
                            if perms.read() { "r" } else { "-" },
                            if perms.write() { "w" } else { "-" },
                            if perms.exec() { "x" } else { "-" },
                        );

                        if let Some(path) = region.path() {
                            println!(" {}", path.display());
                        } else {
                            println!();
                        }
                    }
                } else {
                    println!("{}: dump not found", name);
                }
            } else {
                for name in state.dumps.keys() {
                    println!("{}", name);
                }
            }
        }
    };

    Ok(())
}
