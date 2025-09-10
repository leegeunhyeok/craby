pub mod toolchain {
    pub const TARGETS: &[&str] = &[
        // Android
        "aarch64-linux-android",
        "armv7-linux-androideabi",
        "x86_64-linux-android",
        "i686-linux-android",
        // iOS
        "aarch64-apple-ios",
        "aarch64-apple-ios-sim",
    ];
}

pub mod android {
    pub const ABI_ARM64_V8A: &str = "arm64-v8a";
    pub const ABI_ARMEABI_V7A: &str = "armeabi-v7a";
    pub const ABI_X86_64: &str = "x86_64";
    pub const ABI_X86: &str = "x86";
}

pub mod ios {
    pub const HEADERS_PATH: &str = "Headers";
}
