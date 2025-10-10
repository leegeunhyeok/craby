pub mod toolchain {
    use super::{android::Abi, ios::Identifier};

    pub enum Target {
        Android(Abi),
        Ios(Identifier),
    }

    impl Target {
        pub fn to_str(&self) -> &str {
            match self {
                Target::Android(abi) => match abi {
                    Abi::Arm64V8a => "aarch64-linux-android",
                    Abi::ArmeAbiV7a => "armv7-linux-androideabi",
                    Abi::X86_64 => "x86_64-linux-android",
                    Abi::X86 => "i686-linux-android",
                },
                Target::Ios(identifier) => match identifier {
                    Identifier::Arm64 => "aarch64-apple-ios",
                    Identifier::Arm64Simulator => "aarch64-apple-ios-sim",
                    Identifier::X86_64Simulator => "x86_64-apple-ios",
                    _ => unreachable!(),
                },
            }
        }
    }

    pub const BUILD_TARGETS: [Target; 7] = [
        Target::Android(Abi::Arm64V8a),
        Target::Android(Abi::ArmeAbiV7a),
        Target::Android(Abi::X86_64),
        Target::Android(Abi::X86),
        Target::Ios(Identifier::Arm64),
        Target::Ios(Identifier::Arm64Simulator),
        Target::Ios(Identifier::X86_64Simulator),
    ];
}

pub mod android {
    use std::{collections::HashMap, path::PathBuf};

    use log::debug;

    use crate::platform::android::{get_ndk_clang_path, get_ndk_llvm_ar_path};

    /// See https://github.com/facebook/react-native/blob/v0.74.0/packages/react-native/gradle/libs.versions.toml
    pub const MIN_SDK_VERSION: u8 = 23;

    pub enum Abi {
        Arm64V8a,
        ArmeAbiV7a,
        X86_64,
        X86,
    }

    impl Abi {
        pub fn to_str(&self) -> &str {
            match self {
                Abi::Arm64V8a => "arm64-v8a",
                Abi::ArmeAbiV7a => "armeabi-v7a",
                Abi::X86_64 => "x86_64",
                Abi::X86 => "x86",
            }
        }

        pub fn to_clang_name(&self, cxx: bool) -> String {
            let clang_name = match self {
                Abi::Arm64V8a => "aarch64-linux-android",
                Abi::ArmeAbiV7a => "armv7a-linux-androideabi",
                Abi::X86_64 => "x86_64-linux-android",
                Abi::X86 => "i686-linux-android",
            };

            if cxx {
                format!("{}{}-clang++", clang_name, MIN_SDK_VERSION)
            } else {
                format!("{}{}-clang", clang_name, MIN_SDK_VERSION)
            }
        }

        pub fn to_env(&self) -> Result<HashMap<String, PathBuf>, anyhow::Error> {
            let suffix = match self {
                Abi::Arm64V8a => "aarch64_linux_android",
                Abi::ArmeAbiV7a => "armv7_linux_androideabi",
                Abi::X86_64 => "x86_64_linux_android",
                Abi::X86 => "i686_linux_android",
            };

            let cxxlang_path = get_ndk_clang_path(&self, true)?;
            let clang_path = get_ndk_clang_path(&self, false)?;
            let llvm_ar_path = get_ndk_llvm_ar_path()?;

            let envs = HashMap::from([
                (format!("CXX_{}", suffix), cxxlang_path),
                (format!("CC_{}", suffix), clang_path),
                (format!("AR_{}", suffix), llvm_ar_path),
            ]);

            debug!("Android NDK environments: {:?}", envs);

            Ok(envs)
        }
    }
}

pub mod ios {
    pub enum Identifier {
        /// For device
        Arm64,
        /// For simulator (arm64)
        Arm64Simulator,
        /// For simulator (x86_64)
        X86_64Simulator,
        /// For XCFramework identifier (arm64 + x86_64 architecture for simulator)
        /// Each libraries are combined into a single library by `lipo`
        Simulator,
    }

    impl Identifier {
        pub fn try_into_str(&self) -> Result<&str, anyhow::Error> {
            Ok(match self {
                Identifier::Arm64 => "ios-arm64",
                Identifier::Simulator => "ios-arm64_x86_64-simulator",
                _ => anyhow::bail!("Invalid identifier"),
            })
        }
    }
}
