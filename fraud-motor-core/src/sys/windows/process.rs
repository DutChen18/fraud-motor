use crate::sys::windows::api::*;
use crate::sys::windows::{self, Handle};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{io, iter, mem, ptr, vec};

pub struct List(vec::IntoIter<DWORD>);

pub struct Process(Handle);

pub struct Regions<'a> {
    handle: HANDLE,
    address: Option<SIZE_T>,
    phantom: PhantomData<&'a Process>,
}

pub struct Region {
    info: MEMORY_BASIC_INFORMATION,
    path: Option<PathBuf>,
}

pub struct Permissions(DWORD);

pub struct Memory(Handle);

pub struct Options(DWORD);

impl Iterator for List {
    type Item = io::Result<u32>;

    fn next(&mut self) -> Option<io::Result<u32>> {
        self.0.next().map(Ok)
    }
}

impl Process {
    pub fn open(id: u32) -> io::Result<Process> {
        windows::check(unsafe {
            processthreadsapi::OpenProcess(winnt::PROCESS_QUERY_INFORMATION, minwindef::FALSE, id)
        })
        .map(|handle| Process(Handle(handle)))
    }

    pub fn regions(&self) -> io::Result<Regions> {
        Ok(Regions {
            handle: *self.0,
            address: Some(0),
            phantom: PhantomData,
        })
    }

    pub fn path(&self) -> io::Result<PathBuf> {
        let mut buf = [0; minwindef::MAX_PATH];

        windows::check(unsafe {
            psapi::GetProcessImageFileNameA(*self.0, buf.as_mut_ptr() as LPSTR, buf.len() as DWORD)
        })
        .map(|size| String::from_utf8_lossy(&buf[..size as usize]))
        .map(|string| string.into_owned().into())
    }
}

impl<'a> Iterator for Regions<'a> {
    type Item = io::Result<Region>;

    fn next(&mut self) -> Option<io::Result<Region>> {
        iter::from_fn(|| {
            self.address.map(|address| unsafe {
                let mut info = mem::zeroed();

                windows::check(memoryapi::VirtualQueryEx(
                    self.handle,
                    address as LPCVOID,
                    &mut info,
                    mem::size_of_val(&info),
                ))
                .map(|_| self.address = address.checked_add(info.RegionSize))
                .map(|_| info)
            })
        })
        .take_while(|result| {
            !result.as_ref().is_err_and(|err| {
                err.raw_os_error()
                    .is_some_and(|err| err as DWORD == winerror::ERROR_INVALID_PARAMETER)
            })
        })
        .find(|result| {
            !result
                .as_ref()
                .is_ok_and(|info| info.State & winnt::MEM_COMMIT == 0)
        })
        .map(|result| {
            result.and_then(|info| {
                let path = if info.Type & (winnt::MEM_IMAGE | winnt::MEM_MAPPED) == 0 {
                    None
                } else {
                    let mut buf = [0; minwindef::MAX_PATH];

                    windows::check(unsafe {
                        psapi::GetMappedFileNameA(
                            self.handle,
                            info.BaseAddress,
                            buf.as_mut_ptr() as LPSTR,
                            buf.len() as DWORD,
                        )
                    })
                    .map(|size| String::from_utf8_lossy(&buf[..size as usize]))
                    .map(|string| Some(string.into_owned().into()))?
                };

                Ok(Region { info, path })
            })
        })
    }
}

impl Region {
    pub fn start(&self) -> usize {
        self.info.BaseAddress as usize
    }

    pub fn end(&self) -> usize {
        self.start().wrapping_add(self.info.RegionSize)
    }

    pub fn permissions(&self) -> Permissions {
        Permissions(self.info.Protect)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl Permissions {
    pub fn read(&self) -> bool {
        self.0
            & (winnt::PAGE_EXECUTE_READ
                | winnt::PAGE_EXECUTE_READWRITE
                | winnt::PAGE_EXECUTE_WRITECOPY
                | winnt::PAGE_READONLY
                | winnt::PAGE_READWRITE
                | winnt::PAGE_WRITECOPY)
            != 0
    }

    pub fn write(&self) -> bool {
        self.0
            & (winnt::PAGE_EXECUTE_READWRITE
                | winnt::PAGE_EXECUTE_WRITECOPY
                | winnt::PAGE_READWRITE
                | winnt::PAGE_WRITECOPY)
            != 0
    }

    pub fn execute(&self) -> bool {
        self.0
            & (winnt::PAGE_EXECUTE
                | winnt::PAGE_EXECUTE_READ
                | winnt::PAGE_EXECUTE_READWRITE
                | winnt::PAGE_EXECUTE_WRITECOPY)
            != 0
    }
}

impl Memory {
    pub fn open(id: u32, options: &Options) -> io::Result<Memory> {
        windows::check(unsafe { processthreadsapi::OpenProcess(options.0, minwindef::FALSE, id) })
            .map(|handle| Memory(Handle(handle)))
    }

    pub fn read(&self, buf: &mut [u8], addr: usize) -> io::Result<()> {
        windows::check(unsafe {
            memoryapi::ReadProcessMemory(
                *self.0,
                addr as LPCVOID,
                buf.as_mut_ptr() as LPVOID,
                buf.len(),
                ptr::null_mut(),
            )
        })
        .map(|_| ())
    }

    pub fn write(&self, buf: &[u8], addr: usize) -> io::Result<()> {
        windows::check(unsafe {
            memoryapi::WriteProcessMemory(
                *self.0,
                addr as LPVOID,
                buf.as_ptr() as LPCVOID,
                buf.len(),
                ptr::null_mut(),
            )
        })
        .map(|_| ())
    }
}

impl Options {
    pub fn new() -> Options {
        Options(0)
    }

    pub fn read(&mut self, read: bool) -> &mut Options {
        self.0 &= !winnt::PROCESS_VM_READ;
        self.0 |= winnt::PROCESS_VM_READ * read as DWORD;
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Options {
        self.0 &= !winnt::PROCESS_VM_WRITE;
        self.0 |= winnt::PROCESS_VM_WRITE * write as DWORD;
        self
    }
}

pub fn list() -> io::Result<List> {
    let mut result = vec![0; 1024];

    loop {
        let capacity = (result.len() * mem::size_of::<DWORD>()) as DWORD;
        let mut size = 0;

        windows::check(unsafe { psapi::EnumProcesses(result.as_mut_ptr(), capacity, &mut size) })?;

        if size != capacity {
            result.truncate(size as usize / mem::size_of::<DWORD>());
            break Ok(List(result.into_iter()));
        }

        result.resize(result.len() * 2, 0);
    }
}
