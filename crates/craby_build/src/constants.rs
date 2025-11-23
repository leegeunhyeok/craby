pub mod cxx {
    pub const STD_VERSION: &str = "c++20";
}

pub mod toolchain {
    use std::fmt::Display;

    use super::{android::Abi, ios::Identifier};

    #[derive(Debug, Clone, Copy)]
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

    impl TryFrom<&str> for Target {
        type Error = anyhow::Error;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "aarch64-linux-android" => Ok(Target::Android(Abi::Arm64V8a)),
                "armv7-linux-androideabi" => Ok(Target::Android(Abi::ArmeAbiV7a)),
                "x86_64-linux-android" => Ok(Target::Android(Abi::X86_64)),
                "i686-linux-android" => Ok(Target::Android(Abi::X86)),
                "aarch64-apple-ios" => Ok(Target::Ios(Identifier::Arm64)),
                "aarch64-apple-ios-sim" => Ok(Target::Ios(Identifier::Arm64Simulator)),
                "x86_64-apple-ios" => Ok(Target::Ios(Identifier::X86_64Simulator)),
                _ => anyhow::bail!("Invalid target: {}", value),
            }
        }
    }

    impl Display for Target {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.to_str())
        }
    }

    pub const DEFAULT_ANDROID_TARGETS: [Target; 4] = [
        Target::Android(Abi::Arm64V8a),
        Target::Android(Abi::ArmeAbiV7a),
        Target::Android(Abi::X86_64),
        Target::Android(Abi::X86),
    ];

    pub const DEFAULT_IOS_TARGETS: [Target; 3] = [
        Target::Ios(Identifier::Arm64),
        Target::Ios(Identifier::Arm64Simulator),
        Target::Ios(Identifier::X86_64Simulator),
    ];
}

pub mod android {
    use std::{collections::HashMap, fmt::Display, path::PathBuf};

    use log::debug;

    use crate::platform::android::path::{ndk_clang_path, ndk_llvm_ar_path};

    /// See https://github.com/facebook/react-native/blob/v0.76.0/packages/react-native/gradle/libs.versions.toml
    pub const MIN_SDK_VERSION: u8 = 23;

    #[derive(Debug, Clone, Copy)]
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

            let cxxlang_path = ndk_clang_path(self, true)?;
            let clang_path = ndk_clang_path(self, false)?;
            let llvm_ar_path = ndk_llvm_ar_path()?;

            let envs = HashMap::from([
                (format!("CXX_{}", suffix), cxxlang_path),
                (format!("CC_{}", suffix), clang_path),
                (format!("AR_{}", suffix), llvm_ar_path),
            ]);

            debug!("Android NDK environments: {:?}", envs);

            Ok(envs)
        }
    }

    impl Display for Abi {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.to_str())
        }
    }
}

pub mod ios {
    #[derive(Debug, Clone, Copy)]
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
