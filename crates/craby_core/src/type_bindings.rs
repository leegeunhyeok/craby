#[cfg(feature = "android")]
pub mod android {
    pub mod ffi {

        use jni::sys::{jdouble, jstring};

        pub type Boolean = bool;
        pub type Number = jdouble;
        pub type String = jstring;
    }
}

#[cfg(feature = "ios")]
pub mod ios {
    pub mod ffi {
        use std::os::raw::{c_char, c_double};

        pub type Boolean = bool;
        pub type Number = c_double;
        pub type String = *const c_char;
        pub type Void = ();
    }
}
