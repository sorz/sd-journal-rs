use crate::{
    flags::OpenFlags,
    journal_sys::{
        sd_journal as SdJournal,
        sd_journal_open,
        sd_journal_close,
        sd_journal_get_usage,
        sd_journal_seek_head,
        sd_journal_seek_tail,
        sd_journal_next,
    },
};
use std::marker::PhantomData;

pub type SdResult<T> = Result<T, i32>;

macro_rules! checked_unsafe_call {
    ($e:expr) => {
        let ret = unsafe { $e };
        if ret < 0 {
            return Err(ret);
        }
    };
}


#[derive(Debug)]
struct Journal {
    ret: *mut SdJournal,
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

    pub fn seek_head(&mut self) -> SdResult<()> {
        checked_unsafe_call! {
            sd_journal_seek_head(self.ret)
        }
        Ok(())
    }

    pub fn seek_tail(&mut self) -> SdResult<()> {
        checked_unsafe_call! {
            sd_journal_seek_tail(self.ret)
        }
        Ok(())
    }

    pub fn next(&mut self) -> SdResult<bool> {
        match unsafe { sd_journal_next(self.ret) } {
            0 => Ok(false),
            1 => Ok(true),
            e => Err(e),
        }        
    }
}


#[test]
fn test_open() {
    let mut journal = Journal::open(OpenFlags::CURRENT_USER).unwrap();
    println!("usage {}", journal.usage().unwrap());
    journal.seek_head().unwrap();
    println!("seek head done");
    journal.next().unwrap();
    println!("next head done");

}
