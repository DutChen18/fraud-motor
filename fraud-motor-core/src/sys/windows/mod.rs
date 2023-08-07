pub mod memory;
pub mod process;

mod api {
    pub use winapi::shared::basetsd::*;
    pub use winapi::shared::minwindef::*;
    pub use winapi::shared::winerror::*;
    pub use winapi::um::handleapi::*;
    pub use winapi::um::memoryapi::*;
    pub use winapi::um::processthreadsapi::*;
    pub use winapi::um::psapi::*;
    pub use winapi::um::winnt::*;
}

use std::ops::Deref;
use std::{io, ptr};

trait Error: PartialEq {
    const ERROR: Self;
}

struct Handle(api::HANDLE);

impl Error for api::BOOL {
    const ERROR: api::BOOL = api::FALSE;
}

impl Error for api::SIZE_T {
    const ERROR: api::SIZE_T = 0;
}

impl Error for api::HANDLE {
    const ERROR: api::HANDLE = ptr::null_mut();
}

impl Error for api::DWORD {
    const ERROR: api::DWORD = 0;
}

impl Deref for Handle {
    type Target = api::HANDLE;

    fn deref(&self) -> &api::HANDLE {
        &self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            check(api::CloseHandle(self.0)).unwrap();
        }
    }
}

fn check<T: Error>(result: T) -> io::Result<T> {
    if result != T::ERROR {
        Ok(result)
    } else {
        Err(io::Error::last_os_error())
    }
}
