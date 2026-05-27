// Filename: crates/ble-android-ffi/src/ffi_safety.rs
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug, thiserror::Error)]
pub enum FfiError {
    #[error("null pointer from JNI")]
    NullPtr,
    #[error("invalid UTF-8 from JNI")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}

pub fn cstr_from_ptr(ptr: *const c_char) -> Result<&'static CStr, FfiError> {
    if ptr.is_null() {
        return Err(FfiError::NullPtr);
    }
    // Safety: caller guarantees ptr is a valid, NUL-terminated C string.
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr)
}

pub fn str_from_ptr(ptr: *const c_char) -> Result<String, FfiError> {
    let cstr = cstr_from_ptr(ptr)?;
    Ok(cstr.to_str()?.to_owned())
}
