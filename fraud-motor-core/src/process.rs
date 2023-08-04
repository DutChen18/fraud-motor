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

    pub fn execute(&self) -> bool {
        self.0.execute()
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

    fn find_region(regions: &[Region], addr: usize) -> &Region {
        regions
            .iter()
            .find(|region| region.start() <= addr && addr < region.end())
            .unwrap()
    }

    #[test]
    fn test_list() {
        let ids: Vec<_> = list().unwrap().map(Result::unwrap).collect();

        assert!(ids.contains(&std::process::id()));
        assert!(ids.len() > 1);
    }

    #[test]
    fn test_process_regions() {
        static mut DATA: u32 = 0;
        static RODATA: u32 = 0;
        let stack = 0;
        let heap = Box::leak(Box::new(0));

        let process = Process::open(std::process::id()).unwrap();
        let regions: Vec<_> = process.regions().unwrap().map(Result::unwrap).collect();
        let current_exe = std::env::current_exe().unwrap();
        let file = current_exe.file_name().unwrap();

        let text_region = find_region(&regions, test_process_regions as usize);
        let data_region = find_region(&regions, unsafe { std::ptr::addr_of!(DATA) } as usize);
        let rodata_region = find_region(&regions, std::ptr::addr_of!(RODATA) as usize);
        let stack_region = find_region(&regions, std::ptr::addr_of!(stack) as usize);
        let heap_region = find_region(&regions, std::ptr::addr_of!(*heap) as usize);

        assert!(text_region.permissions().execute() && !text_region.permissions().write());
        assert!(data_region.permissions().read() && data_region.permissions().write());
        assert!(rodata_region.permissions().read() && !rodata_region.permissions().write());
        assert!(stack_region.permissions().read() && stack_region.permissions().write());
        assert!(heap_region.permissions().read() && heap_region.permissions().write());

        assert_eq!(text_region.path().unwrap().file_name().unwrap(), file);
        assert_eq!(data_region.path().unwrap().file_name().unwrap(), file);
        assert_eq!(rodata_region.path().unwrap().file_name().unwrap(), file);
    }

    #[test]
    fn test_process_path() {
        let process = Process::open(std::process::id()).unwrap();
        let process_path = process.path().unwrap();
        let current_exe = std::env::current_exe().unwrap();
        let file = current_exe.file_name().unwrap();

        assert_eq!(process_path.file_name().unwrap(), file);
    }

    #[test]
    fn test_memory_read() {
        let id = std::process::id();
        let memory = Memory::options().read(true).open(id).unwrap();
        let secret: u32 = 1337;
        let mut buf = [0; std::mem::size_of::<u32>()];

        memory
            .read(&mut buf, std::ptr::addr_of!(secret) as usize)
            .unwrap();

        assert_eq!(secret, u32::from_ne_bytes(buf));
    }

    #[test]
    fn test_memory_write() {
        let id = std::process::id();
        let memory = Memory::options().write(true).open(id).unwrap();
        let secret: u32 = 0;
        let buf = u32::to_ne_bytes(1337);

        memory
            .write(&buf, std::ptr::addr_of!(secret) as usize)
            .unwrap();

        assert_eq!(secret, 1337);
    }
}
