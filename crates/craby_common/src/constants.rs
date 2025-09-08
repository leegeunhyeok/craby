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
    pub const ABI_TARGETS: &[&str] = &[
        // Target: aarch64-linux-android
        "arm64-v8a",
        // Target: armv7-linux-androideabi
        "armeabi-v7a",
        // Target: x86_64-linux-android
        "x86_64",
        // Target: i686-linux-android
        "x86"
    ];
}

pub mod ios {}

pub const IMPL_MOD_NAME: &str = "impls";
pub const TEMP_DIR: &str = ".craby";
