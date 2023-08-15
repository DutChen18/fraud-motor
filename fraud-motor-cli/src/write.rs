use crate::state::State;
use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser)]
pub struct Args {
    addr: usize,
    #[command(subcommand)]
    value: Value,
}

#[derive(Subcommand)]
pub enum Value {
    U8 { value: u8 },
    U16 { value: u16 },
    U32 { value: u32 },
    U64 { value: u64 },
    I8 { value: i8 },
    I16 { value: i16 },
    I32 { value: i32 },
    I64 { value: i64 },
    F32 { value: f32 },
    F64 { value: f64 },
}

pub fn write(state: &mut State, args: Args) -> Result<(), Box<dyn Error>> {
    match args.value {
        Value::U8 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::U16 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::U32 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::U64 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::I8 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::I16 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::I32 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::I64 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::F32 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
        Value::F64 { value } => state.memory.write(&value.to_ne_bytes(), args.addr)?,
    };

    Ok(())
}
