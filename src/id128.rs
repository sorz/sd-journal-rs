use crate::{
    SdResult,
    journal_sys::{
        sd_id128_t as SdId128,
        sd_id128_to_string,
        sd_id128_get_machine,
        sd_id128_get_boot,
    },
};
use std::{
    fmt,
    mem,
    os::raw::c_char,
};

#[derive(Copy, Clone)]
pub struct Id128(pub(crate) SdId128);

impl fmt::Display for Id128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; 33];
        unsafe {
            sd_id128_to_string(
                self.0,
                &mut buf as *mut _ as *mut c_char
            );
        }
        let s = std::str::from_utf8(&buf[..32]).expect("invalid utf-8 string");
        f.write_str(s)
    }
}

impl fmt::Debug for Id128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Id128")
         .field(&self.to_string())
         .finish()
    }
}

impl Id128 {
    fn machine() -> SdResult<Self> {
        let mut id = Id128(unsafe { mem::zeroed() });
        checked_unsafe_call! { sd_id128_get_machine(&mut id.0) };
        Ok(id)
    }

    fn boot() -> SdResult<Self> {
        let mut id = Id128(unsafe { mem::zeroed() });
        checked_unsafe_call! { sd_id128_get_boot(&mut id.0) };
        Ok(id)
    }
}

#[test]
fn test_get_id() {
    println!("machine id: {}", Id128::machine().unwrap());
    println!("boot id: {}", Id128::boot().unwrap());
}