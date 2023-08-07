use std::io;

pub struct Memory;

pub struct Options;

impl Memory {
    pub fn open(id: u32, options: &Options) -> io::Result<Memory> {
        unimplemented!()
    }

    pub fn read(&self, buf: &mut [u8], addr: usize) -> io::Result<()> {
        unimplemented!()
    }

    pub fn write(&self, buf: &[u8], addr: usize) -> io::Result<()> {
        unimplemented!()
    }
}

impl Options {
    pub fn new() -> Options {
        unimplemented!()
    }

    pub fn read(&mut self, read: bool) -> &mut Options {
        unimplemented!()
    }

    pub fn write(&mut self, write: bool) -> &mut Options {
        unimplemented!()
    }
}
