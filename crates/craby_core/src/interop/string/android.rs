use jni::objects::JString;
use jni::JNIEnv;

#[derive(Debug)]
pub struct NativeString {
    raw: jni::sys::jstring,
}

impl NativeString {
    pub fn new(raw: jni::sys::jstring) -> Self {
        Self { raw }
    }
}

#[derive(Debug)]
pub enum StringConversionError {
    JniError(jni::errors::Error),
}

impl From<jni::errors::Error> for StringConversionError {
    fn from(err: jni::errors::Error) -> Self {
        StringConversionError::JniError(err)
    }
}

pub trait ToNativeString {
    fn to_native(&self, env: &mut JNIEnv) -> Result<NativeString, StringConversionError>;
}

pub trait FromNativeString: Sized {
    fn from_native(
        native: &NativeString,
        env: &mut jni::JNIEnv,
    ) -> Result<Self, StringConversionError>;
}

impl ToNativeString for String {
    fn to_native(&self, env: &mut JNIEnv) -> Result<NativeString, StringConversionError> {
        let jstring = env.new_string(self)?;
        Ok(NativeString::new(jstring.into_raw()))
    }
}

impl ToNativeString for &str {
    fn to_native(&self, env: &mut JNIEnv) -> Result<NativeString, StringConversionError> {
        let jstring = env.new_string(*self)?;
        Ok(NativeString::new(jstring.into_raw()))
    }
}

impl FromNativeString for String {
    fn from_native(n_str: &NativeString, env: &mut JNIEnv) -> Result<Self, StringConversionError> {
        unsafe {
            let jstring = JString::from_raw(&mut *n_str.raw);
            let java_str = env.get_string(&jstring)?;
            Ok(java_str.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_native() {
        let jvm_args = jni::InitArgsBuilder::new()
            .version(jni::JNIVersion::V8)
            .option("-Xcheck:jni")
            .build()
            .unwrap();

        let jvm = jni::JavaVM::new(jvm_args).unwrap();
        let _guard = jvm.attach_current_thread().unwrap();
        let mut env = jvm.get_env().unwrap();

        let str = "Hello, world!".to_native(&mut env).unwrap();
        println!("str_ptr: {:?}", str.raw);
        assert_eq!(
            std::any::type_name_of_val(&str.raw),
            "*mut jni_sys::_jobject"
        );

        let str = String::from_native(&str, &mut env).unwrap();
        assert_eq!(str, "Hello, world!");
    }
}
