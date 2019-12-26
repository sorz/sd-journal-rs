use crate::journal_sys as sys;
use bitflags::bitflags;


bitflags! {
    pub struct OpenFlags: u32 {
        const LOCAL_ONLY = sys::SD_JOURNAL_LOCAL_ONLY;
        const RUNTIME_ONLY = sys::SD_JOURNAL_RUNTIME_ONLY;
        const SYSTEM = sys::SD_JOURNAL_SYSTEM;
        const CURRENT_USER = sys::SD_JOURNAL_CURRENT_USER;
        const OS_ROOT = sys::SD_JOURNAL_OS_ROOT;
    }
}

impl From<OpenFlags> for i32 {
    fn from(flags: OpenFlags) -> i32 {
        flags.bits() as i32
    }
}