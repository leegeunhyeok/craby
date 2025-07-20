pub mod toolchain {
    pub const TARGETS: &[&str] = &[
        // Android
        "aarch64-linux-android",
        "armv7-linux-androideabi",
        "x86_64-linux-android",
        // iOS
        "aarch64-apple-ios",
        "aarch64-apple-ios-sim",
    ];
}

pub mod android {
    pub const ABI_TARGETS: &[&str] = &["arm64-v8a", "armeabi-v7a", "x86_64"];
}

pub mod ios {}

pub const IMPL_MOD_NAME: &str = "impls";
pub const TEMP_DIR: &str = ".craby";
