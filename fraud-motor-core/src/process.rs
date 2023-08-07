use crate::sys::process as process_imp;
use std::io;
use std::path::{Path, PathBuf};

pub struct List(process_imp::List);

pub struct Process(process_imp::Process);

pub struct Regions<'a>(process_imp::Regions<'a>);

pub struct Region(process_imp::Region);

pub struct Permissions(process_imp::Permissions);

pub struct Memory(process_imp::Memory);

pub struct Options(process_imp::Options);

impl Iterator for List {
    type Item = io::Result<u32>;

    fn next(&mut self) -> Option<io::Result<u32>> {
        self.0.next()
    }
}

impl Process {
    pub fn open(id: u32) -> io::Result<Process> {
        process_imp::Process::open(id).map(Process)
    }

    pub fn regions(&self) -> io::Result<Regions> {
        self.0.regions().map(Regions)
    }

    pub fn path(&self) -> io::Result<PathBuf> {
        self.0.path()
    }
}

impl<'a> Iterator for Regions<'a> {
    type Item = io::Result<Region>;

    fn next(&mut self) -> Option<io::Result<Region>> {
        self.0.next().map(|region| region.map(Region))
    }
}

impl Region {
    pub fn start(&self) -> usize {
        self.0.start()
    }

    pub fn end(&self) -> usize {
        self.0.end()
    }

    pub fn permissions(&self) -> Permissions {
        Permissions(self.0.permissions())
    }

    pub fn path(&self) -> Option<&Path> {
        self.0.path()
    }
}

impl Permissions {
    pub fn read(&self) -> bool {
        self.0.read()
    }

    pub fn write(&self) -> bool {
        self.0.write()
    }

    pub fn exec(&self) -> bool {
        self.0.exec()
    }
}

impl Memory {
    pub fn options() -> Options {
        Options(process_imp::Options::new())
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
        process_imp::Memory::open(id, &self.0).map(Memory)
    }
}

pub fn list() -> io::Result<List> {
    process_imp::list().map(List)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::{env, process, ptr};

    fn find_region(regions: &[Region], addr: usize) -> &Region {
        regions
            .into_iter()
            .find(|region| region.start() <= addr && addr < region.end())
            .unwrap()
    }

    #[test]
    fn test_list() {
        let ids: Vec<_> = list().unwrap().collect::<io::Result<_>>().unwrap();
        let unique_ids: HashSet<_> = ids.iter().copied().collect();

        assert!(ids.contains(&process::id()));
        assert!(ids.len() > 1);
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_process_regions() {
        static mut DATA_VAR: u32 = 0;
        static RODATA_VAR: u32 = 0;
        let stack_var = 0;
        let heap_var = Box::leak(Box::new(0));

        let proc = Process::open(process::id()).unwrap();
        let regions: Vec<_> = proc.regions().unwrap().collect::<io::Result<_>>().unwrap();
        let exe_path = env::current_exe().unwrap();
        let exe_name = exe_path.file_name().unwrap();
        let mut start = 0;

        for region in &regions {
            assert!(region.start() >= start);
            start = region.end();
        }

        let text = find_region(&regions, test_process_regions as usize);
        let data = find_region(&regions, unsafe { ptr::addr_of!(DATA_VAR) } as usize);
        let rodata = find_region(&regions, ptr::addr_of!(RODATA_VAR) as usize);
        let stack = find_region(&regions, ptr::addr_of!(stack_var) as usize);
        let heap = find_region(&regions, ptr::addr_of!(*heap_var) as usize);

        assert!(text.permissions().exec() && !text.permissions().write());
        assert!(data.permissions().read() && data.permissions().write());
        assert!(rodata.permissions().read() && !rodata.permissions().write());
        assert!(stack.permissions().read() && stack.permissions().write());
        assert!(heap.permissions().read() && heap.permissions().write());

        assert_eq!(text.path().and_then(Path::file_name), Some(exe_name));
        assert_eq!(data.path().and_then(Path::file_name), Some(exe_name));
        assert_eq!(rodata.path().and_then(Path::file_name), Some(exe_name));
        assert_eq!(stack.path(), None);
        assert_eq!(heap.path(), None);
    }

    #[test]
    fn test_process_path() {
        let proc = Process::open(process::id()).unwrap();
        let proc_path = proc.path().unwrap();
        let exe_path = env::current_exe().unwrap();
        let exe_name = exe_path.file_name().unwrap();

        assert_eq!(proc_path.file_name(), Some(exe_name));
    }

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
