use std::ffi::{CStr, CString};

use crate::type_bindings::ios;

#[derive(Debug)]
pub enum StringConversionError {
    NullPointer,
    InvalidUtf8,
    CStringCreation,
}

pub trait ToNativeString {
    fn to_native(&self) -> Result<ios::ffi::String, StringConversionError>;
}

pub trait FromNativeString: Sized {
    fn from_native(native: ios::ffi::String) -> Result<Self, StringConversionError>;
}

impl ToNativeString for String {
    fn to_native(&self) -> Result<ios::ffi::String, StringConversionError> {
        let c_string =
            CString::new(self.as_str()).map_err(|_| StringConversionError::CStringCreation)?;
        Ok(c_string.into_raw())
    }
}

impl ToNativeString for &str {
    fn to_native(&self) -> Result<ios::ffi::String, StringConversionError> {
        let c_string = CString::new(*self).map_err(|_| StringConversionError::CStringCreation)?;
        Ok(c_string.into_raw())
    }
}

impl FromNativeString for String {
    fn from_native(str_ptr: ios::ffi::String) -> Result<Self, StringConversionError> {
        if str_ptr.is_null() {
            return Err(StringConversionError::NullPointer);
        }

        unsafe {
            CStr::from_ptr(str_ptr)
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
        println!("str_ptr: {:?}", str);
        assert_eq!(std::any::type_name_of_val(&str), "*const i8");

        let str = String::from_native(str).unwrap();
        assert_eq!(str, "Hello, world!");
    }
}
