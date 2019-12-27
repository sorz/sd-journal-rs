use crate::{journal::Journal, journal_sys::sd_journal_get_data, SdResult};
use std::{
    ffi::{c_void, CString},
    fmt,
};

#[derive(Copy, Clone, Debug)]
pub struct Field<'e> {
    pub name: &'e str,
    pub data: &'e [u8],
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

pub struct Entry<'j> {
    pub(crate) journal: &'j Journal,
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
