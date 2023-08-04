pub mod process;

mod api {
    pub use winapi::shared::basetsd::SIZE_T;
    pub use winapi::shared::minwindef::{self, BOOL, DWORD, LPCVOID, LPVOID};
    pub use winapi::shared::winerror;
    pub use winapi::um::winnt::{self, HANDLE, LPSTR, MEMORY_BASIC_INFORMATION};
    pub use winapi::um::{handleapi, memoryapi, processthreadsapi, psapi};
}

use api::*;
use std::io;
use std::ops::Deref;

trait IsErr {
    fn is_err(&self) -> bool;
}

struct Handle(HANDLE);

impl IsErr for BOOL {
    fn is_err(&self) -> bool {
        *self != minwindef::FALSE
    }
}

impl IsErr for SIZE_T {
    fn is_err(&self) -> bool {
        *self != 0
    }
}

impl IsErr for HANDLE {
    fn is_err(&self) -> bool {
        !self.is_null()
    }
}

impl IsErr for DWORD {
    fn is_err(&self) -> bool {
        *self != 0
    }
}

impl Deref for Handle {
    type Target = HANDLE;

    fn deref(&self) -> &HANDLE {
        &self.0
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        check(unsafe { handleapi::CloseHandle(self.0) }).unwrap();
    }
}

fn check<T: IsErr>(result: T) -> io::Result<T> {
    if result.is_err() {
        Ok(result)
    } else {
        Err(io::Error::last_os_error())
    }
}
