pub struct Scan {
    regions: Vec<(usize, Box<[u8]>)>,
    align: usize,
    len: usize,
}

pub struct Iter<'a> {
    regions: &'a [(usize, Box<[u8]>)],
    align: usize,
    addr: usize,
    data: &'a [u8],
    byte: u8,
    bit: u32,
}

impl Scan {
    pub fn new(align: usize) -> Scan {
        Scan {
            regions: Vec::new(),
            align,
            len: 0,
        }
    }

    pub fn insert(&mut self, start: usize, end: usize) {
        let size = (end - start + self.align - 1) / self.align;
        let mut data = vec![255; (size + 7) / 8].into_boxed_slice();

        if size % 8 != 0 {
            data[size / 8] >>= 8 - size % 8;
        }

        self.regions.push((start, data));
        self.len += size;
    }

    pub fn retain<F: FnMut(usize) -> bool>(&mut self, mut filter: F) {
        for &mut (mut addr, ref mut data) in self.regions.iter_mut() {
            for byte in data.iter_mut() {
                if *byte != 0 {
                    for bit in 0..8 {
                        let mask = 1 << bit;

                        if *byte & mask != 0 && !filter(addr) {
                            *byte &= !mask;
                            self.len -= 1;
                        }

                        addr += self.align;
                    }
                } else {
                    addr += self.align * 8;
                }
            }
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            regions: &self.regions,
            align: self.align,
            addr: 0,
            data: &[],
            byte: 0,
            bit: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        'a: loop {
            while self.bit < 8 {
                let mask = 1 << self.bit;
                let addr = self.addr;

                self.bit += 1;
                self.addr += self.align;

                if self.byte & mask != 0 {
                    return Some(addr);
                }
            }

            while let Some((&byte, data)) = self.data.split_first() {
                self.data = data;

                if byte != 0 {
                    self.bit = 0;
                    self.byte = byte;

                    continue 'a;
                }

                self.addr += self.align * 8;
            }

            let (&(addr, ref data), scan) = self.regions.split_first()?;

            self.addr = addr;
            self.data = data.as_ref();
            self.regions = scan;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dump::ProcessDump;
    use crate::memory::Memory;
    use crate::process::{Process, Region};
    use std::{process, ptr};

    fn region_filter(region: &Region, addr: usize) -> bool {
        region.permissions().write() && addr >= region.start() && addr < region.end()
    }

    #[test]
    fn test_scan() {
        let id = process::id();
        let memory = Memory::options().read(true).open(id).unwrap();
        let proc = Process::open(id).unwrap();
        let mut scan = Scan::new(4);
        let mut size = 0;
        let mut secret = Box::new(0);
        let secret_addr = ptr::addr_of!(*secret) as usize;

        assert_eq!(scan.len(), 0);
        assert!(scan.is_empty());

        for region in proc.regions().unwrap() {
            let region = region.unwrap();

            if region_filter(&region, secret_addr) {
                scan.insert(region.start(), region.end());
                size += (region.end() - region.start() + 3) / 4;
            }
        }

        assert_eq!(scan.len(), size);
        assert!(!scan.is_empty());

        loop {
            let dump =
                ProcessDump::new(&memory, &proc, |region| region_filter(region, secret_addr))
                    .unwrap();

            let mut view = dump.view();
            let last_len = scan.len();

            scan.retain(|addr| {
                view.data(addr)
                    .and_then(|buf| buf.get(..4))
                    .map(|buf| u32::from_ne_bytes(buf.try_into().unwrap()))
                    == Some(*secret)
            });

            assert!(scan.len() <= last_len);
            assert!(scan.iter().any(|addr| addr == secret_addr));

            if scan.len() == last_len {
                break;
            }

            *secret += 1;
        }

        assert!(scan.len() <= 2);
        assert!(!scan.is_empty());
    }
}
