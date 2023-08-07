use crate::sys::windows::{self, api, Handle};
use std::{io, ptr};

pub struct Memory(Handle);

pub struct Options(api::DWORD);

impl Memory {
    pub fn open(id: u32, options: &Options) -> io::Result<Memory> {
        unsafe {
            let handle = windows::check(api::OpenProcess(options.0, api::FALSE, id))?;

            Ok(Memory(Handle(handle)))
        }
    }

    pub fn read(&self, buf: &mut [u8], addr: usize) -> io::Result<()> {
        unsafe {
            windows::check(api::ReadProcessMemory(
                *self.0,
                addr as api::LPCVOID,
                buf.as_mut_ptr() as api::LPVOID,
                buf.len(),
                ptr::null_mut(),
            ))?;

            Ok(())
        }
    }

    pub fn write(&self, buf: &[u8], addr: usize) -> io::Result<()> {
        unsafe {
            windows::check(api::WriteProcessMemory(
                *self.0,
                addr as api::LPVOID,
                buf.as_ptr() as api::LPCVOID,
                buf.len(),
                ptr::null_mut(),
            ))?;

            Ok(())
        }
    }
}

impl Options {
    pub fn new() -> Options {
        Options(0)
    }

    pub fn read(&mut self, read: bool) -> &mut Options {
        self.0 &= !api::PROCESS_VM_READ;
        self.0 |= api::PROCESS_VM_READ * read as api::DWORD;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Options {
        self.0 &= !api::PROCESS_VM_WRITE;
        self.0 |= api::PROCESS_VM_WRITE * write as api::DWORD;
        self
    }
}
