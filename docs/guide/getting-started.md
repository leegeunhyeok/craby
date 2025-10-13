# Getting Started

:::warning
This project is currently under development and is in early alpha. For more information about the stable release, please refer to the following [link](https://github.com/leegeunhyeok/craby/issues/1)
:::

This guide will walk you through creating your first Craby module from scratch.

## Compatibility

Craby-built modules require a minimum React Native version of:

- `>= 0.76.0` (**with New Architecture enabled**)

## Prerequisites

::: warning macOS Required
Craby development requires **macOS** with **Xcode 12 or higher** for building [*-apple-ios](https://doc.rust-lang.org/rustc/platform-support/apple-ios.html) targets.
:::

Before you begin, make sure you have the following installed:

- **macOS**: Required for building iOS targets
- **XCode 12 or higher**: [Download](https://developer.apple.com/xcode)
- **Node.js 18+**: [Download](https://nodejs.org/)
- **Rust**: Install via [rustup](https://rustup.rs/)
- **Android NDK**: [Download](https://developer.android.com/ndk/downloads)

```bash
# Set `ANDROID_NDK_HOME` environment variable
export ANDROID_NDK_HOME=/path/to/android-ndk
```

::: info

You can use the `doctor` command to verify that all requirements are met.

```bash
npx crabygen doctor
```

:::

## Installation

You have two options for getting started with Craby: scaffolding a new module or adding it manually to an existing project.

### Option 1: Scaffold a New Module (Recommended)

The quickest way to get started is to use the `crabygen init` command:

```bash
npx crabygen init <module-name>
cd <module-name>
```

This will create a complete module structure with:
- Rust workspace configuration
- Native build setup (Android/iOS)
- Package configuration

### Option 2: Manual Installation

If you want to add Craby to an existing React Native module:

::: code-group
```bash [npm]
npm install craby-modules
npm install --save-dev crabygen
```

```bash [pnpm]
pnpm add craby-modules
pnpm add -D crabygen
```

```bash [yarn]
yarn add craby-modules
yarn add -D crabygen
```
:::

After installation, you'll need to set up the project structure manually (see [Project Structure](#project-structure) below).

### iOS Configuration

In your `.podspec` file:

```ruby
Pod::Spec.new do |s|
  s.name         = "YourModule"
  s.version      = "1.0.0"

  # Include C++ and iOS source files
  s.source_files = ["ios/**/*.{h,m,mm,cc,cpp}", "cpp/**/*.{hpp,cpp}"]
  s.private_header_files = "ios/include/*.h"

  # Link the XCFramework
  s.vendored_frameworks = "ios/framework/libyourmodule.xcframework"

  # Add flag to using C++20
  s.pod_target_xcconfig = {
    "CLANG_CXX_LANGUAGE_STANDARD" => "c++20",
  }
end
```

### Android Configuration

In `android/build.gradle`:

```groovy
def reactNativeArchitectures() {
  def value = rootProject.getProperties().get("reactNativeArchitectures")
  return value ? value.split(",") : ["armeabi-v7a", "x86", "x86_64", "arm64-v8a"]
}

// Configure CMake
android {
  defaultConfig {
    externalNativeBuild {
      cmake {
        targets "cxx-my-module"
        cppFlags "-frtti -fexceptions -Wall -Wextra -fstack-protector-all"
        arguments "-DANDROID_STL=c++_shared", "-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON"
        abiFilters (*reactNativeArchitectures())
        buildTypes {
          debug {
            cppFlags "-O1 -g"
          }
          release {
            cppFlags "-O2"
          }
        }
      }
    }
  }

  externalNativeBuild {
    cmake {
      path "CMakeLists.txt"
    }
  }

  buildTypes {
    debug {
      jniDebuggable true
    }
    release {
      minifyEnabled false
      externalNativeBuild {
        cmake {
          arguments "-DCMAKE_BUILD_TYPE=Release"
        }
      }
    }
  }
}
```

## Project Structure

After scaffolding or setup, your project will have this structure:

```
your-module/
├── src/                          # TypeScript source
│   ├── index.ts                  # Module exports
│   └── NativeModule.ts           # TurboModule spec
├── crates/                       # Rust workspace
│   └── lib/
│       ├── Cargo.toml
│       ├── build.rs
│       └── src/
│           ├── lib.rs            # Module entry
│           ├── module_impl.rs    # Your implementation ⭐
│           ├── ffi.rs            # Generated FFI layer
│           ├── types.rs          # Helper types
│           └── generated.rs      # Generated traits
├── cpp/                          # Pure C++ TurboModule code
├── android/                      # Android native setup
│   ├── build.gradle
│   └── CMakeLists.txt
├── ios/                          # iOS native setup
│   └── framework/                # Generated XCFramework
├── Cargo.toml                    # Root Cargo workspace
├── rust-toolchain.toml           # Rust version config
└── package.json
```

## Your First Module

Let's create a simple calculator module to understand the Craby workflow.

### Step 1: Define the TypeScript Spec

Create `src/NativeCalculator.ts`:

::: info
Module spec files must start with the "Native" prefix.
:::

```typescript
import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface Spec extends NativeModule {
  add(a: number, b: number): number;
  subtract(a: number, b: number): number;
  multiply(a: number, b: number): number;
  divide(a: number, b: number): number;
}

export default NativeModuleRegistry.getEnforcing<Spec>('Calculator');
```

Export your module in `src/index.ts`:

```typescript
export { default as Calculator } from './NativeCalculator';
```

### Step 2: Generate Code

Run the code generation command:

```bash
npx crabygen
```

### Step 3: Implement the Rust Logic

Open `crates/lib/src/calculator_impl.rs` and implement the trait:

```rust
use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct Calculator {
    id: usize,
}

impl CalculatorSpec for Calculator {
    fn new(id: usize) -> Self {
        Calculator { id }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn add(&mut self, a: Number, b: Number) -> Number {
        a + b
    }

    fn subtract(&mut self, a: Number, b: Number) -> Number {
        a - b
    }

    fn multiply(&mut self, a: Number, b: Number) -> Number {
        a * b
    }

    fn divide(&mut self, a: Number, b: Number) -> Number {
        a / b
    }
}
```

### Step 4: Build Native Binaries

Build the Rust code for all target platforms:

```bash
npx crabygen build
```

### Step 5: Sync with Native Projects

- Android: Sync Gradle
- iOS: Install Pods

### Step 6: Build the React Native Application and Run

Now you can use your module in your React Native app:

```typescript
import { Calculator } from 'your-module';

const sum = Calculator.add(10, 5); // 15
const difference = Calculator.subtract(10, 5); // 5
const product = Calculator.multiply(10, 5); // 50
const quotient = Calculator.divide(10, 5); // 2
```

## Next Steps

Now that you've created your first module, explore:

- [Module Definition](/guide/module-definition) - Learn about module specs and types
- [How to build](/guide/build) - Build native binaries
