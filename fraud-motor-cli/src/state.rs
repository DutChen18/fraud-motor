use fraud_motor_core::dump::ProcessDump;
use fraud_motor_core::memory::Memory;
use fraud_motor_core::process::Process;
use fraud_motor_core::scan::Scan;
use std::collections::HashMap;
use std::error::Error;

pub struct State {
    pub memory: Memory,
    pub proc: Process,
    pub dumps: HashMap<String, ProcessDump>,
    pub scans: HashMap<String, ScanGroup>,
}

pub struct ScanGroup {
    pub u8: Option<Scan>,
    pub u16: Option<Scan>,
    pub u32: Option<Scan>,
    pub u64: Option<Scan>,
    pub i8: Option<Scan>,
    pub i16: Option<Scan>,
    pub i32: Option<Scan>,
    pub i64: Option<Scan>,
    pub f32: Option<Scan>,
    pub f64: Option<Scan>,
}

impl State {
    pub fn new(pid: u32) -> Result<State, Box<dyn Error>> {
        Ok(State {
            memory: Memory::options().read(true).write(true).open(pid)?,
            proc: Process::open(pid)?,
            dumps: HashMap::new(),
            scans: HashMap::new(),
        })
    }
}
