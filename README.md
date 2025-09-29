<div align="center">

<img width="360" src="./logo.png" alt="logo">

# craby

Type-safe Rust for React Native—auto generated, integrated with pure C++ TurboModule

</div>

> [!NOTE]
> This project is under development
>
> Follow [this issue](https://github.com/leegeunhyeok/craby/issues/1) for updates on the upcoming stable release.

## Overview

**Craby** is a type-safe Rust development tool for React Native. It automatically generates Rust/C++ code based on TypeScript schemas and is fully integrated with **pure C++ TurboModule** (No platform interop such as `ObjCTurboModule` and `JavaTurboModule`).

### Key Features

- ⚡️ **Blazing Fast**: Integrated with pure C++ TurboModule
- 🔄 **Auto Code Generation**: Automatically generate Rust/C++ code from TurboModule schemas
- 🛡️ **Type Safety**: Prevent runtime errors with compile-time type validation
- 🔧 **Developer Experience**: Simple CLI commands for project setup and building

## Documentation

TBD

## Quick Start

### Prerequisites

- [XCode 12 or higher](https://developer.apple.com/xcode) for [iOS targets](https://doc.rust-lang.org/rustc/platform-support/apple-ios.html) (macOS required)
- [Android NDK](https://developer.android.com/ndk/downloads) and `ANDROID_NDK_HOME` environment variable

### Scaffold

```bash
npx crabygen init <moduleName>
```

### Manual Installation

```bash
# NPM
npm install cragy-modules
npm install --dev crabygen

# pnpm
pnpm install cragy-modules
pnpm install --dev crabygen

# Yarn
yarn add cray-modules
yarn add --dev crabygen
```

## Commands

### `crabygen codegen`

> Alias of `crabygen` command

Generates Rust/C++ code based on your TypeScript module schemas. This command:

- Analyzes your NativeModule spec files
- Generates corresponding Rust function signatures and C++ bridging implementations
- Generates native bindings for Android(CMakefile) and iOS(XCFramework)

### `crabygen build`

Compiles Rust code and generates native binaries.

### `crabygen show`

Displays project information and schemas.

### `crabygen doctor`

Checks project configuration and dependencies.

### `crabygen clean`

Cleans up build caches and generated temporary files.

## Project Structure

After running `crabygen init`, your project structure will look like this:

```
your-turbo-module/
├── src/
│   ├── index.ts
│   └── NativeModule.ts
├── crates/
│   └── lib/
│       ├── Cargo.toml
│       ├── build.rs
│       └── src/
│           ├── lib.rs
│           ├── {name}_impl.rs.rs # Your Rust implementation ⭐️
│           ├── ffi.rs            # FFI Layer for C++
│           ├── types.rs          # Helper types for Rust
│           └── generated.rs      # Module specifications (Trait)
├── cpp/                          # Pure C++ TurboModules
├── android/
├── ios/
├── Cargo.toml                    # Root Cargo workspace
├── rust-toolchain.toml           # Rust toolchain configuration
└── package.json
```

## Examples

```typescript
// src/NativeModule.ts
import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface Spec extends NativeModule {
  add(a: number, b: number): number;
  subtract(a: number, b: number): number;
}

export default NativeModuleRegistry.getEnforcing<Spec>('Calculator');
```

After running `crabygen`, you'll get:

```rust,ignore
// crates/lib/src/generated.rs (auto-generated)
#[rustfmt::skip]
use crate::ffi::bridging::*;
use crate::types::*;

pub trait CalculatorSpec {
    fn new(id: usize) -> Self;
    fn id(&self) -> usize;
    fn add(a: f64, b: f64) -> f64;
    fn subtract(a: f64, b: f64) -> f64;
}
```

```rust,ignore
// crates/lib/src/calculator_impl.rs
use crate::ffi::bridging::*;
use crate::generated::*;
use crate::types::*;

pub struct Calculator {
    id: usize,
}

impl Calculator for CalculatorSpec {
    fn new(id: usize) -> Calculator {
        Calculator { id }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn add(a: f64, b: f64) -> f64 {
        unimplemented!(); // Implement here!
    }
    
    fn subtract(a: f64, b: f64) -> f64 {
        unimplemented!(); // Implement here!
    }
}
```

## Manual Setup

### Android

Open `android/build.gradle` file and add the following line:

```java
android {
  externalNativeBuild {
    // Add CMake build configuration
    cmake {
      path "CMakeLists.txt"
      targets "cxx-calculator"

      // ...
    }
  }
}
```

### iOS

Open `<ModuleName>.podspec` file and add the following line:

```rb
Pod::Spec.new do |s|
  # Add these lines
  s.source_files = ["ios/**/*.{{h,m,mm,cc,cpp}}", "cpp/**/*.{{hpp,cpp}}"]
  s.private_header_files = "ios/include/*.h"
  s.vendored_frameworks = "ios/framework/libcalculator.xcframework"
end
```

## Development

### Requirements

- Node.js 18+
- Rust (nightly-2025-08-04)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
