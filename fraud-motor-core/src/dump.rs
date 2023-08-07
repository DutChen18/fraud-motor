use crate::process::{Memory, Process, Region};
use std::io;
use std::sync::{Arc, Mutex, Weak};

pub struct RegionDump(Arc<[u8]>);

pub struct ProcessDump(Box<[(Region, io::Result<RegionDump>)]>);

static REGION_DUMP_POOL: Mutex<Vec<Weak<[u8]>>> = Mutex::new(Vec::new());

impl RegionDump {
    pub fn new(memory: &Memory, region: &Region) -> io::Result<RegionDump> {
        let mut buf = vec![0; region.end() - region.start()];

        memory.read(&mut buf, region.start())?;

        let mut guard = REGION_DUMP_POOL.lock().unwrap();
        let mut pool: Vec<_> = guard.iter().filter_map(Weak::upgrade).collect();

        let arc = match pool.binary_search_by(|dump| dump.as_ref().cmp(&buf)) {
            Ok(i) => Arc::clone(&pool[i]),
            Err(i) => {
                pool.insert(i, buf.into());
                Arc::clone(&pool[i])
            }
        };

        *guard = pool.iter().map(Arc::downgrade).collect();
        Ok(RegionDump(arc))
    }

    pub fn data(&self) -> &[u8] {
        &self.0
    }
}

impl ProcessDump {
    pub fn new<F: FnMut(&Region) -> bool>(
        memory: &Memory,
        process: &Process,
        mut filter: F,
    ) -> io::Result<ProcessDump> {
        let regions = process
            .regions()?
            .filter(|region| !region.as_ref().is_ok_and(|region| !filter(region)))
            .map(|region| {
                let region = region?;
                let dump = RegionDump::new(memory, &region);

                Ok((region, dump))
            })
            .collect::<io::Result<_>>()?;

        Ok(ProcessDump(regions))
    }

    pub fn regions(&self) -> &[(Region, io::Result<RegionDump>)] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    fn any_permissions(region: &Region) -> bool {
        region.permissions().read() || region.permissions().write() || region.permissions().exec()
    }

    #[test]
    fn test_region_dump() {
        let id = process::id();
        let memory = Memory::options().read(true).open(id).unwrap();
        let proc = Process::open(id).unwrap();

        for region in proc.regions().unwrap() {
            let region = region.unwrap();

            if any_permissions(&region) {
                let dump_0 = RegionDump::new(&memory, &region);
                let dump_1 = RegionDump::new(&memory, &region);

                if let Ok(dump0) = dump_0 {
                    let dump1 = dump_1.unwrap();

                    if dump0.data().as_ptr() != dump1.data().as_ptr() {
                        assert_ne!(dump0.data(), dump1.data());
                        assert!(!region.permissions().exec());
                    }
                } else {
                    assert!(dump_1.is_err());
                }
            }
        }
    }

    #[test]
    fn test_process_dump() {
        let id = process::id();
        let memory = Memory::options().read(true).open(id).unwrap();
        let proc = Process::open(id).unwrap();
        let dump = ProcessDump::new(&memory, &proc, any_permissions).unwrap();

        for (region, dump) in dump.regions() {
            assert!(any_permissions(&region));

            if let Ok(dump) = dump {
                assert_eq!(region.end() - region.start(), dump.data().len());
            }
        }
    }
}
