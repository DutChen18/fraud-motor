use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub struct List;

pub struct Process;

pub struct Regions<'a> {
    phantom: PhantomData<&'a Process>,
}

pub struct Region;

pub struct Permissions;

pub struct Memory;

pub struct Options;

impl Iterator for List {
    type Item = io::Result<u32>;

    fn next(&mut self) -> Option<io::Result<u32>> {
        unimplemented!()
    }
}

impl Process {
    pub fn open(id: u32) -> io::Result<Process> {
        unimplemented!()
    }

    pub fn regions(&self) -> io::Result<Regions> {
        unimplemented!()
    }

    pub fn path(&self) -> io::Result<PathBuf> {
        unimplemented!()
    }
}

impl<'a> Iterator for Regions<'a> {
    type Item = io::Result<Region>;

    fn next(&mut self) -> Option<io::Result<Region>> {
        unimplemented!()
    }
}

impl Region {
    pub fn start(&self) -> usize {
        unimplemented!()
    }

    pub fn end(&self) -> usize {
        unimplemented!()
    }

    pub fn permissions(&self) -> Permissions {
        unimplemented!()
    }

    pub fn path(&self) -> Option<&Path> {
        unimplemented!()
    }
}

impl Permissions {
    pub fn read(&self) -> bool {
        unimplemented!()
    }

    pub fn write(&self) -> bool {
        unimplemented!()
    }

    pub fn execute(&self) -> bool {
        unimplemented!()
    }
}

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

pub fn list() -> io::Result<List> {
    unimplemented!()
}
