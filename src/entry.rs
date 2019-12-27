use crate::{journal::Journal, 
    journal_sys::{
        sd_journal_get_data,
        sd_journal_enumerate_data,
        sd_journal_restart_data,
    },
    SdResult
};
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

impl<'j> Entry<'j> {
    pub fn field<S: AsRef<str>>(&mut self, name: S) -> SdResult<Field> {
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

    pub fn fields<'e>(&'e mut self) -> Fields<'e, 'j> {
        Fields(self)
    }
}

pub struct Fields<'e, 'j: 'e>(&'e Entry<'j>);

impl Drop for Fields<'_, '_> {
    fn drop(&mut self) {
        unsafe { sd_journal_restart_data(self.0.journal.ret) }
    }
}

impl<'e, 'j> Iterator for Fields<'e, 'j> {
    type Item = Field<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = 0 as *const u8;
        let mut size = 0usize;
        let ret = unsafe {
            sd_journal_enumerate_data(
                self.0.journal.ret,
                &mut buf as *mut _ as *mut *const c_void,
                &mut size,
            )
        };
        match ret {
            0 => None,
            e if e < 0 => panic!("enumerate fail: {}", e),
            _ => {
                let buf = unsafe { std::slice::from_raw_parts(buf, size) };
                Some(Field::from_raw(buf))
            }
        }
    }
}

#[test]
fn test_enumerate_fields() {
    use crate::flags::OpenFlags;
    use crate::journal::Seek;

    let mut journal = Journal::open(OpenFlags::empty()).unwrap();
    journal.seek(Seek::Head).unwrap();
    assert!(journal.next().unwrap());
    let mut entry = journal.entry();
    for field in entry.fields() {
        println!("field {}", field);
    }
}
