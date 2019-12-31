use crate::{journal::Journal, 
    journal_sys::{
        ENOENT,
        sd_journal_get_data,
        sd_journal_enumerate_data,
        sd_journal_restart_data,
    },
};
use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::{c_void, CString},
    fmt,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldData<'e> {
    pub name: Cow<'e, str>,
    pub data: Cow<'e, [u8]>,
}

impl<'a> FieldData<'a> {
    fn from_raw(data: &'a [u8]) -> Self {
        let mut name_data = data.splitn(2, |c| *c == b'=');
        let name = name_data.next().unwrap();
        let data = name_data.next().expect("missing field name");
        let name = std::str::from_utf8(name).expect("invalid utf-8 field name");
        Self {
            name: Cow::Borrowed(name),
            data: Cow::Borrowed(data),
        }
    }

    pub fn into_owned(self) -> FieldData<'static> {
        FieldData {
            name: Cow::Owned(self.name.into_owned()),
            data: Cow::Owned(self.data.into_owned()),
        }
    }
}

impl fmt::Display for FieldData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, String::from_utf8_lossy(&self.data))
    }
}

pub struct Entry<'j> {
    pub(crate) journal: &'j Journal,
}

impl<'j> Entry<'j> {
    pub fn field<S: AsRef<str>>(&mut self, name: S) -> Option<FieldData> {
        let c_name = CString::new(name.as_ref()).unwrap();
        let mut buf = 0 as *const u8;
        let mut size = 0usize;

        let ret = unsafe {
            sd_journal_get_data(
                self.journal.ret,
                c_name.as_ptr(),
                &mut buf as *mut _ as *mut *const c_void,
                &mut size,
            )
        };
        if ret >= 0 {
            let buf = unsafe { std::slice::from_raw_parts(buf, size) };
            let field = FieldData::from_raw(buf);
            assert_eq!(name.as_ref(), field.name);
            Some(field)
        } else if ret == -(ENOENT as i32) {
            None
        } else {
            panic!("error on get field data: {}", ret);
        }
    }

    pub fn fields<'e>(&'e mut self) -> Fields<'e, 'j> {
        Fields(self)
    }

    pub fn all_fields(&mut self) -> HashMap<String, Vec<u8>> {
        let mut kvs = HashMap::new();
        for field in self.fields() {
            kvs.insert(field.name.into_owned(), field.data.into_owned());
        }
        kvs
    }
}

pub struct Fields<'e, 'j: 'e>(&'e Entry<'j>);

impl Drop for Fields<'_, '_> {
    fn drop(&mut self) {
        unsafe { sd_journal_restart_data(self.0.journal.ret) }
    }
}

impl<'e, 'j> Iterator for Fields<'e, 'j> {
    type Item = FieldData<'e>;

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
                Some(FieldData::from_raw(buf))
            }
        }
    }
}


#[test]
fn test_nonexist_field() {
    use crate::flags::OpenFlags;
    use crate::journal::Seek;

    let mut journal = Journal::open(OpenFlags::empty()).unwrap();
    journal.seek(Seek::Head).unwrap();
    assert!(journal.next().unwrap());
    let mut entry = journal.entry();
    assert!(entry.field("NON_EXIST__").is_none())
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
