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

    pub fn exec(&self) -> bool {
        unimplemented!()
    }
}

pub fn list() -> io::Result<List> {
    unimplemented!()
}
