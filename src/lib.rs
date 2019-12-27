macro_rules! checked_unsafe_call {
    ($e:expr) => {
        let ret = unsafe { $e };
        if ret < 0 {
            return Err(ret);
        }
    };
}

pub mod entry;
pub mod flags;
pub mod id128;
pub mod journal;
mod journal_sys;

pub type SdResult<T> = Result<T, i32>;
