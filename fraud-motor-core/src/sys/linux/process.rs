use std::fs::{self, File, ReadDir};
use std::io::{self, BufRead, BufReader, Lines};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub struct List(ReadDir);

pub struct Process(u32);

pub struct Regions<'a> {
    file: Lines<BufReader<File>>,
    phantom: PhantomData<&'a Process>,
}

pub struct Region {
    start: usize,
    end: usize,
    permissions: Permissions,
    path: Option<PathBuf>,
}

pub struct Permissions {
    read: bool,
    write: bool,
    exec: bool,
}

impl Iterator for List {
    type Item = io::Result<u32>;

    fn next(&mut self) -> Option<io::Result<u32>> {
        self.0.find_map(|entry| {
            entry
                .map(|entry| entry.file_name().to_str()?.parse().ok())
                .transpose()
        })
    }
}

impl Process {
    pub fn open(id: u32) -> io::Result<Process> {
        Ok(Process(id))
    }

    pub fn regions(&self) -> io::Result<Regions> {
        let file = File::open(format!("/proc/{}/maps", self.0))?;

        Ok(Regions {
            file: BufReader::new(file).lines(),
            phantom: PhantomData,
        })
    }

    pub fn path(&self) -> io::Result<PathBuf> {
        fs::read_link(format!("/proc/{}/exe", self.0))
    }
}

impl<'a> Iterator for Regions<'a> {
    type Item = io::Result<Region>;

    fn next(&mut self) -> Option<io::Result<Region>> {
        self.file.next().map(|line| {
            let line = line?;
            let mut line = line.split_whitespace();
            let (start, end) = line.next().unwrap().split_once('-').unwrap();
            let permissions = line.next().unwrap().as_bytes();
            let path = line.nth(3).filter(|line| line.starts_with('/'));

            Ok(Region {
                start: usize::from_str_radix(start, 16).unwrap(),
                end: usize::from_str_radix(end, 16).unwrap(),
                permissions: Permissions {
                    read: permissions[0] == b'r',
                    write: permissions[1] == b'w',
                    exec: permissions[2] == b'x',
                },
                path: path.map(Into::into),
            })
        })
    }
}

impl Region {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn permissions(&self) -> Permissions {
        Permissions { ..self.permissions }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl Permissions {
    pub fn read(&self) -> bool {
        self.read
    }

    pub fn write(&self) -> bool {
        self.write
    }

    pub fn exec(&self) -> bool {
        self.exec
    }
}

pub fn list() -> io::Result<List> {
    fs::read_dir("/proc").map(List)
}
