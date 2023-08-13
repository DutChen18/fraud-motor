use clap::{Parser, Subcommand};
use fraud_motor_core::dump::ProcessDump;
use fraud_motor_core::memory::Memory;
use fraud_motor_core::process::Process;
use fraud_motor_core::scan::Scan;
use rustyline::DefaultEditor;
use std::collections::HashMap;
use std::error::Error;

struct State {
    memory: Memory,
    proc: Process,
    dumps: HashMap<String, ProcessDump>,
    scans: HashMap<String, Scan>,
}

#[derive(Parser)]
struct Args {
    pid: u32,
}

#[derive(Parser)]
struct DumpArgs {
    #[command(subcommand)]
    command: DumpCommands,
}

#[derive(Subcommand)]
enum DumpCommands {
    New { name: String },
    Drop { name: String },
    List,
}

#[derive(Parser)]
struct ScanArgs {
    #[command(subcommand)]
    command: ScanCommands,
}

#[derive(Subcommand)]
enum ScanCommands {
    New {
        name: String,
    },
    Drop {
        name: String,
    },
    List {
        name: Option<String>,
    },
    Next {
        name: String,
        dump: String,
        min: f64,
        max: f64,
    },
}

#[derive(Parser)]
struct WriteArgs {
    addr: usize,
    value: f64,
}

fn dump(state: &mut State, args: DumpArgs) -> Result<(), Box<dyn Error>> {
    match args.command {
        DumpCommands::New { name } => {
            let dump = ProcessDump::new(&state.memory, &state.proc, |region| {
                region.permissions().write()
            })?;

            state.dumps.insert(name, dump);
        }
        DumpCommands::Drop { name } => {
            state.dumps.remove(&name);
        }
        DumpCommands::List => {
            for (name, _) in &state.dumps {
                println!("{}", name);
            }
        }
    };

    Ok(())
}

fn scan(state: &mut State, args: ScanArgs) -> Result<(), Box<dyn Error>> {
    match args.command {
        ScanCommands::New { name } => {
            let mut scan = Scan::new(1);

            for region in state.proc.regions()? {
                let region = region?;

                if region.permissions().write() {
                    scan.insert(region.start(), region.end());
                }
            }

            state.scans.insert(name, scan);
        }
        ScanCommands::Drop { name } => {
            state.scans.remove(&name);
        }
        ScanCommands::List { name } => {
            if let Some(name) = name {
                if let Some(scan) = state.scans.get_mut(&name) {
                    for addr in scan.iter() {
                        println!("{}", addr);
                    }
                } else {
                    println!("{}: scan not found", name);
                }
            } else {
                for (name, scan) in &state.scans {
                    println!("{} ({})", name, scan.len());
                }
            }
        }
        ScanCommands::Next {
            name,
            dump,
            min,
            max,
        } => {
            if let Some(scan) = state.scans.get_mut(&name) {
                if let Some(dump) = state.dumps.get(&dump) {
                    let mut view = dump.view();

                    scan.retain(|addr| {
                        view.data(addr)
                            .and_then(|buf| buf.get(..8))
                            .map(|buf| f64::from_ne_bytes(buf.try_into().unwrap()))
                            .is_some_and(|value| value >= min && value <= max)
                    });
                } else {
                    println!("{}: dump not found", dump);
                }
            } else {
                println!("{}: scan not found", name);
            }
        }
    };

    Ok(())
}

fn write(state: &mut State, args: WriteArgs) -> Result<(), Box<dyn Error>> {
    state
        .memory
        .write(&f64::to_ne_bytes(args.value), args.addr)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut rl = DefaultEditor::new()?;

    let mut state = State {
        memory: Memory::options().read(true).write(true).open(args.pid)?,
        proc: Process::open(args.pid)?,
        dumps: HashMap::new(),
        scans: HashMap::new(),
    };

    loop {
        match rl.readline("(fm) ") {
            Ok(line) => {
                let cmd: Vec<_> = line.split_whitespace().collect();

                match cmd.first() {
                    Some(&"exit") => break,
                    Some(&"dump") => match DumpArgs::try_parse_from(&cmd) {
                        Ok(args) => dump(&mut state, args)?,
                        Err(err) => err.print()?,
                    },
                    Some(&"scan") => match ScanArgs::try_parse_from(&cmd) {
                        Ok(args) => scan(&mut state, args)?,
                        Err(err) => err.print()?,
                    },
                    Some(&"write") => match WriteArgs::try_parse_from(&cmd) {
                        Ok(args) => write(&mut state, args)?,
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
