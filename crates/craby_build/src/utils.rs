pub mod android {
    use crate::constants::android::{ABI_ARM64_V8A, ABI_ARMEABI_V7A, ABI_X86, ABI_X86_64};

    pub fn get_abi_by_target(target: &str) -> &str {
        match target {
            "aarch64-linux-android" => ABI_ARM64_V8A,
            "armv7-linux-androideabi" => ABI_ARMEABI_V7A,
            "x86_64-linux-android" => ABI_X86_64,
            "i686-linux-android" => ABI_X86,
            _ => unreachable!("Unsupported target: {}", target),
        }
    }
}
