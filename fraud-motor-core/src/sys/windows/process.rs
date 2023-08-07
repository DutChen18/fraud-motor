use crate::sys::windows::{self, api, Handle};
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::path::{Path, PathBuf};
use std::{io, iter, slice, str, vec};

pub struct List(vec::IntoIter<api::DWORD>);

pub struct Process(Handle);

pub struct Regions<'a> {
    handle: api::HANDLE,
    addr: Option<api::SIZE_T>,
    phantom: PhantomData<&'a Process>,
}

pub struct Region {
    info: api::MEMORY_BASIC_INFORMATION,
    path: Option<PathBuf>,
}

pub struct Permissions(api::DWORD);

impl Iterator for List {
    type Item = io::Result<u32>;

    fn next(&mut self) -> Option<io::Result<u32>> {
        self.0.next().map(Ok)
    }
}

impl Process {
    pub fn open(id: u32) -> io::Result<Process> {
        unsafe {
            let handle = windows::check(api::OpenProcess(
                api::PROCESS_QUERY_INFORMATION,
                api::FALSE,
                id,
            ))?;

            Ok(Process(Handle(handle)))
        }
    }

    pub fn regions(&self) -> io::Result<Regions> {
        Ok(Regions {
            handle: *self.0,
            addr: Some(0),
            phantom: PhantomData,
        })
    }

    pub fn path(&self) -> io::Result<PathBuf> {
        unsafe {
            let mut buf: [MaybeUninit<u8>; api::MAX_PATH] = MaybeUninit::uninit().assume_init();

            let size = windows::check(api::GetProcessImageFileNameA(
                *self.0,
                buf.as_mut_ptr() as api::LPSTR,
                buf.len() as api::DWORD,
            ))? as usize;

            let buf = slice::from_raw_parts(buf.as_ptr() as *const u8, size);

            Ok(str::from_utf8_unchecked(buf).into())
        }
    }
}

impl<'a> Iterator for Regions<'a> {
    type Item = io::Result<Region>;

    fn next(&mut self) -> Option<io::Result<Region>> {
        iter::from_fn(|| {
            self.addr.map(|addr| unsafe {
                let mut info = MaybeUninit::uninit();

                windows::check(api::VirtualQueryEx(
                    self.handle,
                    addr as api::LPCVOID,
                    info.as_mut_ptr(),
                    mem::size_of_val(&info),
                ))?;

                let info = info.assume_init();

                self.addr = addr.checked_add(info.RegionSize);
                io::Result::Ok(info)
            })
        })
        .take_while(|info| {
            info.as_ref().err().and_then(io::Error::raw_os_error)
                != Some(api::ERROR_INVALID_PARAMETER as i32)
        })
        .map(|info| unsafe {
            let info = info?;

            if info.State & api::MEM_COMMIT == 0 {
                Ok(None)
            } else if info.Type & (api::MEM_IMAGE | api::MEM_MAPPED) == 0 {
                Ok(Some(Region { info, path: None }))
            } else {
                let mut buf: [MaybeUninit<u8>; api::MAX_PATH] = MaybeUninit::uninit().assume_init();

                let size = windows::check(api::GetMappedFileNameA(
                    self.handle,
                    info.BaseAddress,
                    buf.as_mut_ptr() as api::LPSTR,
                    buf.len() as api::DWORD,
                ))? as usize;

                let buf = slice::from_raw_parts(buf.as_ptr() as *const u8, size);
                let path = Some(str::from_utf8_unchecked(buf).into());

                Ok(Some(Region { info, path }))
            }
        })
        .find_map(Result::transpose)
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
    const EXEC_WRITE: api::DWORD = api::PAGE_EXECUTE_READWRITE | api::PAGE_EXECUTE_WRITECOPY;
    const EXEC_READ: api::DWORD = api::PAGE_EXECUTE_READ | Permissions::EXEC_WRITE;
    const EXEC: api::DWORD = api::PAGE_EXECUTE | Permissions::EXEC_READ;
    const WRITE: api::DWORD = api::PAGE_READWRITE | api::PAGE_WRITECOPY | Permissions::EXEC_WRITE;
    const READ: api::DWORD = api::PAGE_READONLY | Permissions::WRITE | Permissions::EXEC_READ;

    pub fn read(&self) -> bool {
        self.0 & Permissions::READ != 0
    }

    pub fn write(&self) -> bool {
        self.0 & Permissions::WRITE != 0
    }

    pub fn exec(&self) -> bool {
        self.0 & Permissions::EXEC != 0
    }
}

pub fn list() -> io::Result<List> {
    unsafe {
        let mut vec = vec![0; 1024];

        loop {
            let capacity = (vec.len() * mem::size_of::<api::DWORD>()) as api::DWORD;
            let mut size = 0;

            windows::check(api::EnumProcesses(vec.as_mut_ptr(), capacity, &mut size))?;

            if size != capacity {
                vec.truncate(size as usize / mem::size_of::<api::DWORD>());

                break Ok(List(vec.into_iter()));
            }

            vec.resize(vec.len() * 2, 0);
        }
    }
}
