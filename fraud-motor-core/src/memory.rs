use crate::sys::memory as memory_imp;
use std::io;

pub struct Memory(memory_imp::Memory);

pub struct Options(memory_imp::Options);

impl Memory {
    pub fn options() -> Options {
        Options(memory_imp::Options::new())
    }

    pub fn read(&self, buf: &mut [u8], addr: usize) -> io::Result<()> {
        self.0.read(buf, addr)
    }

    pub fn write(&self, buf: &[u8], addr: usize) -> io::Result<()> {
        self.0.write(buf, addr)
    }
}

impl Options {
    pub fn read(&mut self, read: bool) -> &mut Options {
        self.0.read(read);
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Options {
        self.0.write(write);
        self
    }

    pub fn open(&self, id: u32) -> io::Result<Memory> {
        memory_imp::Memory::open(id, &self.0).map(Memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{process, ptr};

    #[test]
    fn test_memory_read() {
        let memory = Memory::options().read(true).open(process::id()).unwrap();
        let mut buf = u32::to_ne_bytes(0);
        let secret = u32::to_ne_bytes(1337);
        let addr = ptr::addr_of!(secret) as usize;

        memory.read(&mut buf, addr).unwrap();
        assert!(memory.read(&mut buf, 0).is_err());
        assert_eq!(secret, buf);
    }

    #[test]
    fn test_memory_write() {
        let memory = Memory::options().write(true).open(process::id()).unwrap();
        let buf = u32::to_ne_bytes(1337);
        let secret = u32::to_ne_bytes(0);
        let addr = ptr::addr_of!(secret) as usize;

        memory.write(&buf, addr).unwrap();
        assert!(memory.write(&buf, 0).is_err());
        assert_eq!(secret, buf);
    }
}
