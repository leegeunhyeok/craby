use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug)]
pub struct NativeString {
    raw: *mut c_char,
}

impl NativeString {
    pub fn new(raw: *mut c_char) -> Self {
        Self { raw }
    }
}

#[derive(Debug)]
pub enum StringConversionError {
    NullPointer,
    InvalidUtf8,
    CStringCreation,
}

pub trait ToNativeString {
    fn to_native(&self) -> Result<NativeString, StringConversionError>;
}

pub trait FromNativeString: Sized {
    fn from_native(native: &NativeString) -> Result<Self, StringConversionError>;
}

impl ToNativeString for String {
    fn to_native(&self) -> Result<NativeString, StringConversionError> {
        let c_string =
            CString::new(self.as_str()).map_err(|_| StringConversionError::CStringCreation)?;
        Ok(NativeString::new(c_string.into_raw()))
    }
}

impl ToNativeString for &str {
    fn to_native(&self) -> Result<NativeString, StringConversionError> {
        let c_string = CString::new(*self).map_err(|_| StringConversionError::CStringCreation)?;
        Ok(NativeString::new(c_string.into_raw()))
    }
}

impl FromNativeString for String {
    fn from_native(native: &NativeString) -> Result<Self, StringConversionError> {
        let c_str_ptr: *const i8 = native.raw;

        if c_str_ptr.is_null() {
            return Err(StringConversionError::NullPointer);
        }

        unsafe {
            CStr::from_ptr(c_str_ptr)
                .to_str()
                .map(|s| s.to_owned())
                .map_err(|_| StringConversionError::InvalidUtf8)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_native() {
        let str = "Hello, world!".to_native().unwrap();
        println!("str_ptr: {:?}", str.raw);
        assert_eq!(std::any::type_name_of_val(&str.raw), "*mut i8");

        let str = String::from_native(&str).unwrap();
        assert_eq!(str, "Hello, world!");
    }
}
