use crate::state::{ScanGroup, State};
use clap::{Parser, Subcommand};
use fraud_motor_core::dump::{DumpView, ProcessDump};
use fraud_motor_core::memory::Memory;
use fraud_motor_core::process::Region;
use fraud_motor_core::scan::Scan;
use std::error::Error;
use std::fmt::Display;
use std::mem;
use std::str::FromStr;

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New {
        name: String,
        #[command(flatten)]
        types: Types,
        #[arg(long)]
        align: Option<usize>,
    },
    Drop {
        name: String,
    },
    Info {
        name: Option<String>,
    },
    Next {
        name: String,
        dump: Option<String>,
        #[command(flatten)]
        filters: Filters,
    },
}

#[derive(clap::Args)]
struct Types {
    #[arg(long)]
    u8: bool,
    #[arg(long)]
    u16: bool,
    #[arg(long)]
    u32: bool,
    #[arg(long)]
    u64: bool,
    #[arg(long)]
    i8: bool,
    #[arg(long)]
    i16: bool,
    #[arg(long)]
    i32: bool,
    #[arg(long)]
    i64: bool,
    #[arg(long)]
    f32: bool,
    #[arg(long)]
    f64: bool,
}

#[derive(clap::Args)]
struct Filters {
    #[arg(long)]
    eq: Vec<String>,
    #[arg(long)]
    ne: Vec<String>,
    #[arg(long)]
    gt: Vec<String>,
    #[arg(long)]
    ge: Vec<String>,
    #[arg(long)]
    lt: Vec<String>,
    #[arg(long)]
    le: Vec<String>,
}

fn scan_new(flag: bool, align: usize) -> Option<Scan> {
    if flag {
        Some(Scan::new(align))
    } else {
        None
    }
}

fn scan_insert(scan: Option<&mut Scan>, region: &Region) {
    if let Some(scan) = scan {
        scan.insert(region.start(), region.end());
    }
}

fn scan_info<T, Cvt, const N: usize>(scan: Option<&Scan>, memory: &Memory, ty: &str, mut cvt: Cvt)
where
    T: Display,
    Cvt: FnMut([u8; N]) -> T,
{
    if let Some(scan) = scan {
        for addr in scan.iter() {
            print!("{}:{}", ty, addr);

            let mut buf = [0; N];

            if memory.read(&mut buf, addr).is_ok() {
                println!(" {}", cvt(buf));
            } else {
                println!();
            }
        }
    }
}

fn scan_info_name(scan: Option<&Scan>, name: &str, ty: &str) {
    if let Some(scan) = scan {
        println!("{}:{} {}", name, ty, scan.len());
    }
}

fn scan_next_imp<T, Cvt, Cmp, const N: usize>(
    scan: &mut Scan,
    view: &mut DumpView,
    filter: &[String],
    mut cvt: Cvt,
    mut cmp: Cmp,
) where
    T: FromStr + Copy,
    T::Err: Error + 'static,
    Cvt: FnMut([u8; N]) -> T,
    Cmp: FnMut(T, T) -> bool,
{
    for expr in filter {
        if let Ok(rhs) = expr.parse() {
            scan.retain(|addr| {
                view.data(addr)
                    .and_then(|buf| buf.get(..mem::size_of::<T>()))
                    .is_some_and(|buf| cmp(cvt(buf.try_into().unwrap()), rhs))
            });
        } else {
            scan.retain(|_| false);
        }
    }
}

fn scan_next<T, Cvt, const N: usize>(
    scan: Option<&mut Scan>,
    view: &mut DumpView,
    filters: &Filters,
    cvt: Cvt,
) where
    T: PartialEq + PartialOrd + FromStr + Copy,
    T::Err: Error + 'static,
    Cvt: FnMut([u8; N]) -> T + Copy,
{
    if let Some(scan) = scan {
        scan_next_imp(scan, view, &filters.eq, cvt, |a, b| a == b);
        scan_next_imp(scan, view, &filters.ne, cvt, |a, b| a != b);
        scan_next_imp(scan, view, &filters.gt, cvt, |a, b| a > b);
        scan_next_imp(scan, view, &filters.ge, cvt, |a, b| a >= b);
        scan_next_imp(scan, view, &filters.lt, cvt, |a, b| a < b);
        scan_next_imp(scan, view, &filters.le, cvt, |a, b| a <= b);
    }
}

pub fn scan(state: &mut State, args: Args) -> Result<(), Box<dyn Error>> {
    match args.command {
        Commands::New { name, types, align } => {
            let all = !types.u8
                && !types.u16
                && !types.u32
                && !types.u64
                && !types.i8
                && !types.i16
                && !types.i32
                && !types.i64
                && !types.f32
                && !types.f64;

            let mut scan = ScanGroup {
                u8: scan_new(types.u8 || all, align.unwrap_or(1)),
                u16: scan_new(types.u16 || all, align.unwrap_or(2)),
                u32: scan_new(types.u32 || all, align.unwrap_or(4)),
                u64: scan_new(types.u64 || all, align.unwrap_or(8)),
                i8: scan_new(types.i8 || all, align.unwrap_or(1)),
                i16: scan_new(types.i16 || all, align.unwrap_or(2)),
                i32: scan_new(types.i32 || all, align.unwrap_or(4)),
                i64: scan_new(types.i64 || all, align.unwrap_or(8)),
                f32: scan_new(types.f32 || all, align.unwrap_or(4)),
                f64: scan_new(types.f64 || all, align.unwrap_or(8)),
            };

            for region in state.proc.regions()? {
                let region = region?;

                if region.permissions().write() {
                    scan_insert(scan.u8.as_mut(), &region);
                    scan_insert(scan.u16.as_mut(), &region);
                    scan_insert(scan.u32.as_mut(), &region);
                    scan_insert(scan.u64.as_mut(), &region);
                    scan_insert(scan.i8.as_mut(), &region);
                    scan_insert(scan.i16.as_mut(), &region);
                    scan_insert(scan.i32.as_mut(), &region);
                    scan_insert(scan.i64.as_mut(), &region);
                    scan_insert(scan.f32.as_mut(), &region);
                    scan_insert(scan.f64.as_mut(), &region);
                }
            }

            state.scans.insert(name, scan);
        }
        Commands::Drop { name } => {
            state.scans.remove(&name);
        }
        Commands::Info { name } => {
            if let Some(name) = name {
                if let Some(scan) = state.scans.get(&name) {
                    scan_info(scan.u8.as_ref(), &state.memory, "u8", u8::from_ne_bytes);
                    scan_info(scan.u16.as_ref(), &state.memory, "u16", u16::from_ne_bytes);
                    scan_info(scan.u32.as_ref(), &state.memory, "u32", u32::from_ne_bytes);
                    scan_info(scan.u64.as_ref(), &state.memory, "u64", u64::from_ne_bytes);
                    scan_info(scan.i8.as_ref(), &state.memory, "i8", i8::from_ne_bytes);
                    scan_info(scan.i16.as_ref(), &state.memory, "i16", i16::from_ne_bytes);
                    scan_info(scan.i32.as_ref(), &state.memory, "i32", i32::from_ne_bytes);
                    scan_info(scan.i64.as_ref(), &state.memory, "i64", i64::from_ne_bytes);
                    scan_info(scan.f32.as_ref(), &state.memory, "f32", f32::from_ne_bytes);
                    scan_info(scan.f64.as_ref(), &state.memory, "f64", f64::from_ne_bytes);
                } else {
                    println!("{}: scan not found", name);
                }
            } else {
                for (name, scan) in &state.scans {
                    scan_info_name(scan.u8.as_ref(), name, "u8");
                    scan_info_name(scan.u16.as_ref(), name, "u16");
                    scan_info_name(scan.u32.as_ref(), name, "u32");
                    scan_info_name(scan.u64.as_ref(), name, "u64");
                    scan_info_name(scan.i8.as_ref(), name, "i8");
                    scan_info_name(scan.i16.as_ref(), name, "i16");
                    scan_info_name(scan.i32.as_ref(), name, "i32");
                    scan_info_name(scan.i64.as_ref(), name, "i64");
                    scan_info_name(scan.f32.as_ref(), name, "f32");
                    scan_info_name(scan.f64.as_ref(), name, "f64");
                }
            }
        }
        Commands::Next {
            name,
            dump,
            filters,
        } => {
            if let Some(scan) = state.scans.get_mut(&name) {
                let tmp_dump;

                let mut view = if let Some(dump) = dump {
                    if let Some(dump) = state.dumps.get(&dump) {
                        dump.view()
                    } else {
                        println!("{}: dump not found", dump);

                        return Ok(());
                    }
                } else {
                    tmp_dump = ProcessDump::new(&state.memory, &state.proc, |region| {
                        region.permissions().write()
                    })?;

                    tmp_dump.view()
                };

                scan_next(scan.u8.as_mut(), &mut view, &filters, u8::from_ne_bytes);
                scan_next(scan.u16.as_mut(), &mut view, &filters, u16::from_ne_bytes);
                scan_next(scan.u32.as_mut(), &mut view, &filters, u32::from_ne_bytes);
                scan_next(scan.u64.as_mut(), &mut view, &filters, u64::from_ne_bytes);
                scan_next(scan.i8.as_mut(), &mut view, &filters, i8::from_ne_bytes);
                scan_next(scan.i16.as_mut(), &mut view, &filters, i16::from_ne_bytes);
                scan_next(scan.i32.as_mut(), &mut view, &filters, i32::from_ne_bytes);
                scan_next(scan.i64.as_mut(), &mut view, &filters, i64::from_ne_bytes);
                scan_next(scan.f32.as_mut(), &mut view, &filters, f32::from_ne_bytes);
                scan_next(scan.f64.as_mut(), &mut view, &filters, f64::from_ne_bytes);
            } else {
                println!("{}: scan not found", name);
            }
        }
    };

    Ok(())
}
