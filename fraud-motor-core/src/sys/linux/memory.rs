use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::fs::FileExt;

pub struct Memory(File);

pub struct Options(OpenOptions);

impl Memory {
    pub fn open(id: u32, options: &Options) -> io::Result<Memory> {
        options.0.open(format!("/proc/{}/mem", id)).map(Memory)
    }

    pub fn read(&self, buf: &mut [u8], addr: usize) -> io::Result<()> {
        self.0.read_exact_at(buf, addr as u64)
    }

    pub fn write(&self, buf: &[u8], addr: usize) -> io::Result<()> {
        self.0.write_all_at(buf, addr as u64)
    }
}

impl Options {
    pub fn new() -> Options {
        Options(OpenOptions::new())
    }

    pub fn read(&mut self, read: bool) -> &mut Options {
        self.0.read(read);
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Options {
        self.0.write(write);
        self
    }
}
