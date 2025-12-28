use std::*;

pub unsafe fn as_bytes<T>(v: &T) -> &[u8] {
    unsafe { slice::from_raw_parts(v as *const T as *const u8, mem::size_of::<T>()) }
}
