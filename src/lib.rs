macro_rules! checked_unsafe_call {
    ($e:expr) => {
        let ret = unsafe { $e };
        if ret < 0 {
            return Err(ret);
        }
    };
}

mod journal_sys;
pub mod journal;
pub mod flags;
pub mod id128;

pub type SdResult<T> = Result<T, i32>;
