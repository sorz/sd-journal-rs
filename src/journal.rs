use crate::{
    SdResult,
    flags::OpenFlags,
    id128::Id128,
    journal_sys::{
        sd_journal as SdJournal,
        sd_journal_open,
        sd_journal_close,
        sd_journal_get_usage,
        sd_journal_seek_head,
        sd_journal_seek_tail,
        sd_journal_seek_monotonic_usec,
        sd_journal_seek_realtime_usec,
        sd_journal_next,
        sd_journal_previous,
        sd_journal_next_skip,
        sd_journal_previous_skip,
        sd_journal_get_data,
    },
};
use std::{
    fmt,
    ffi::{c_void, CString},
};

#[derive(Debug, Clone)]
pub enum Seek {
    Head,
    Tail,
    Monotonic {
        boot_id: Id128,
        usec: u64,
    },
    Realtime {
        usec: u64,
    }
}

#[derive(Debug)]
pub struct Journal {
    ret: *mut SdJournal,
}

impl Drop for Journal {
    fn drop(&mut self) {
        unsafe {
            sd_journal_close(self.ret);
        }
    }
}

pub struct Entry<'j> {
    journal: &'j Journal,
}

#[derive(Copy, Clone, Debug)]
pub struct Field<'e> {
    pub name: &'e str,
    pub data: &'e [u8],
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
}

impl Entry<'_> {
    pub fn field<S: AsRef<str>>(&self, name: S) -> SdResult<Field> {
        let c_name = CString::new(name.as_ref()).unwrap();
        let mut buf = 0 as *const u8;
        let mut size = 0usize;

        checked_unsafe_call! {
            sd_journal_get_data(
                self.journal.ret,
                c_name.as_ptr(),
                &mut buf as *mut _ as *mut *const c_void,
                &mut size,
            )
        };
        let buf = unsafe { std::slice::from_raw_parts(buf, size) };
        let field = Field::from_raw(buf);
        assert_eq!(name.as_ref(), field.name);
        Ok(field)
    }
}

impl<'a> Field<'a> {
    fn from_raw(data: &'a [u8]) -> Self {
        let mut name_data = data.splitn(2, |c| *c == b'=');
        let name = name_data.next().unwrap();
        let data = name_data.next().expect("missing field name");
        let name = std::str::from_utf8(name).expect("invalid utf-8 field name");
        Field { name, data }
    }
}

impl fmt::Display for Field<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, String::from_utf8_lossy(self.data))
    }
}

#[test]
fn test_open() {
    let mut journal = Journal::open(OpenFlags::empty()).unwrap();
    println!("usage {}", journal.usage().unwrap());
    journal.seek(Seek::Head).unwrap();
    println!("seek head done");
    assert!(journal.next().unwrap());
    println!("next head done");
    let entry = journal.entry();
    println!("field {}", entry.field("MESSAGE_ID").unwrap());

}
