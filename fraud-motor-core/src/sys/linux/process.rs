use std::fs::{self, File, OpenOptions, ReadDir};
use std::io::{self, BufRead, BufReader, Lines};
use std::marker::PhantomData;
use std::os::unix::fs::FileExt;
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
    execute: bool,
}

pub struct Memory(File);

pub struct Options(OpenOptions);

impl Iterator for List {
    type Item = io::Result<u32>;

    fn next(&mut self) -> Option<io::Result<u32>> {
        self.0
            .by_ref()
            .map(|entry| entry.map(|entry| entry.file_name().into_string().ok()?.parse().ok()))
            .find_map(Result::transpose)
    }
}

impl Process {
    pub fn open(id: u32) -> io::Result<Process> {
        Ok(Process(id))
    }

    pub fn regions(&self) -> io::Result<Regions> {
        let path = format!("/proc/{}/maps", self.0);

        File::open(path).map(|file| Regions {
            file: BufReader::new(file).lines(),
            phantom: PhantomData,
        })
    }

    pub fn path(&self) -> io::Result<PathBuf> {
        let path = format!("/proc/{}/exe", self.0);

        fs::read_link(path)
    }
}

impl<'a> Iterator for Regions<'a> {
    type Item = io::Result<Region>;

    fn next(&mut self) -> Option<io::Result<Region>> {
        self.file.next().map(|line| {
            line.map(|line| {
                let mut line = line.split_whitespace();
                let (start, end) = line.next().unwrap().split_once('-').unwrap();
                let permissions = line.next().unwrap().as_bytes();
                let path = line.nth(3).filter(|line| line.starts_with('/'));

                Region {
                    start: usize::from_str_radix(start, 16).unwrap(),
                    end: usize::from_str_radix(end, 16).unwrap(),
                    permissions: Permissions {
                        read: permissions[0] == b'r',
                        write: permissions[1] == b'w',
                        execute: permissions[2] == b'x',
                    },
                    path: path.map(Into::into),
                }
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

    pub fn execute(&self) -> bool {
        self.execute
    }
}

impl Memory {
    pub fn open(id: u32, options: &Options) -> io::Result<Memory> {
        let path = format!("/proc/{}/mem", id);

        options.0.open(path).map(Memory)
    }

    pub fn read(&self, buf: &mut [u8], addr: usize) -> io::Result<()> {
        self.0.read_exact_at(buf, addr as u64)
    }

    pub fn write(&self, buf: &[u8], addr: usize) -> io::Result<()> {
        self.0.write_all_at(buf, addr as u64)
    }
}

impl Options {
    pub fn new() -> Options {
        Options(OpenOptions::new())
    }

    pub fn read(&mut self, read: bool) -> &mut Options {
        self.0.read(read);
        self
    }

    pub fn write(&mut self, write: bool) -> &mut Options {
        self.0.write(write);
        self
    }
}

pub fn list() -> io::Result<List> {
    fs::read_dir("/proc").map(List)
}
