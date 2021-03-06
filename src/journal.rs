use crate::{
    entry::Entry,
    flags::OpenFlags,
    id128::Id128,
    journal_sys::{
        sd_journal as SdJournal, sd_journal_close, sd_journal_get_usage, sd_journal_next,
        sd_journal_next_skip, sd_journal_open, sd_journal_previous, sd_journal_previous_skip,
        sd_journal_seek_head, sd_journal_seek_monotonic_usec, sd_journal_seek_realtime_usec,
        sd_journal_seek_tail, sd_journal_enumerate_fields, sd_journal_restart_fields
    },
    SdResult,
};
use std::{
    ffi::CStr,
    os::raw::c_char,
};

#[derive(Debug, Clone)]
pub enum Seek {
    Head,
    Tail,
    Monotonic { boot_id: Id128, usec: u64 },
    Realtime { usec: u64 },
}

#[derive(Debug)]
pub struct Journal {
    pub(crate) ret: *mut SdJournal,
}

impl Drop for Journal {
    fn drop(&mut self) {
        unsafe {
            sd_journal_close(self.ret);
        }
    }
}

impl Journal {
    pub fn open(flags: OpenFlags) -> SdResult<Self> {
        let mut ret = 0 as *mut SdJournal;
        checked_unsafe_call! {
            sd_journal_open(
                (&mut ret) as *mut _ as *mut *mut SdJournal,
                flags.into(),
            )
        }
        Ok(Self { ret })
    }

    pub fn usage(&self) -> SdResult<u64> {
        let mut bytes = 0u64;
        checked_unsafe_call! {
            sd_journal_get_usage(
                self.ret,
                (&mut bytes) as *mut u64,
            )
        }
        Ok(bytes)
    }

    pub fn seek(&mut self, pos: Seek) -> SdResult<()> {
        checked_unsafe_call! {
            match pos {
                Seek::Head => sd_journal_seek_head(self.ret),
                Seek::Tail => sd_journal_seek_tail(self.ret),
                Seek::Realtime { usec } =>
                    sd_journal_seek_realtime_usec(self.ret, usec),
                Seek::Monotonic { boot_id, usec } =>
                    sd_journal_seek_monotonic_usec(self.ret, boot_id.0, usec)
            }
        };
        Ok(())
    }

    pub fn next(&mut self) -> SdResult<bool> {
        match unsafe { sd_journal_next(self.ret) } {
            0 => Ok(false),
            1 => Ok(true),
            e => Err(e),
        }
    }

    pub fn previous(&mut self) -> SdResult<bool> {
        match unsafe { sd_journal_previous(self.ret) } {
            0 => Ok(false),
            1 => Ok(true),
            e => Err(e),
        }
    }

    pub fn skip(&mut self, n: i64) -> SdResult<u64> {
        let ret = if n >= 0 {
            unsafe { sd_journal_next_skip(self.ret, n as u64) }
        } else {
            unsafe { sd_journal_previous_skip(self.ret, -n as u64) }
        };
        if ret < 0 {
            Err(ret)
        } else {
            Ok(ret as u64)
        }
    }

    pub fn entry(&mut self) -> Entry {
        Entry { journal: self }
    }

    pub fn next_entry(&mut self) -> Option<Entry> {
        match self.next() {
            Ok(true) => Some(self.entry()),
            Ok(false) => None,
            Err(e) => unimplemented!("error on next(): {}", e),
        }
    }

    pub fn previous_entry(&mut self) -> Option<Entry> {
        match self.previous() {
            Ok(true) => Some(self.entry()),
            Ok(false) => None,
            Err(e) => unimplemented!("error on previous(): {}", e),
        }
    }

    pub fn all_fields(&mut self) -> Vec<String> {
        unsafe { sd_journal_restart_fields(self.ret) };
        let mut fields = Vec::new();
        let mut field_ptr = 0 as *const c_char;
        loop {
            match unsafe {
                sd_journal_enumerate_fields(self.ret, &mut field_ptr)
            } {
                0 => break,
                e if e < 0 => panic!("error: sd_journal_enumerate_fields() return {}", e),
                _ => {
                    let cstr = unsafe { CStr::from_ptr(field_ptr) };
                    let field = cstr.to_string_lossy().into_owned();
                    fields.push(field);
                }
            }
        }
        fields
    }
}

#[test]
fn test_get_all_fields() {
    let mut journal = Journal::open(OpenFlags::empty()).unwrap();
    let fields = journal.all_fields();
    println!("fields: {:?}", fields);
}

#[test]
fn test_next_previous_netry() {
    let mut journal = Journal::open(OpenFlags::empty()).unwrap();
    let id1 = journal.next_entry().unwrap().field("MESSAGE").unwrap().into_owned();
    let id2 = journal.next_entry().unwrap().field("MESSAGE").unwrap().into_owned();
    let _id3 = journal.next_entry().unwrap().field("MESSAGE").unwrap().into_owned();
    let id2p = journal.previous_entry().unwrap().field("MESSAGE").unwrap().into_owned();
    let id1p = journal.previous_entry().unwrap().field("MESSAGE").unwrap().into_owned();
    assert_eq!(id1, id1p);
    assert_eq!(id2, id2p);
}

#[test]
fn test_open() {
    let mut journal = Journal::open(OpenFlags::empty()).unwrap();
    println!("usage {}", journal.usage().unwrap());
    journal.seek(Seek::Head).unwrap();
    println!("seek head done");
    assert!(journal.next().unwrap());
    println!("next head done");
    let mut entry = journal.entry();
    println!("field {}", entry.field("MESSAGE").unwrap());
}
