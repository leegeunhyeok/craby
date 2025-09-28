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

**Craby** is a type-safe Rust development tool for React Native. It automatically generates Rust/C++ code based on TypeScript/Flow schemas and is fully integrated with **pure C++ TurboModule** (No platform interop such as `ObjCTurboModule` and `JavaTurboModule`).

### Key Features

- ⚡️ **Blazing Fast**: Integrated with pure C++ TurboModule
- 🔄 **Auto Code Generation**: Automatically generate Rust/C++ code from TurboModule schemas
- 🛡️ **Type Safety**: Prevent runtime errors with compile-time type validation
- 🔧 **Developer Experience**: Simple CLI commands for project setup and building

## Quick Start

### Prerequisites

- [XCode 12 or higher](https://developer.apple.com/xcode) for [iOS targets](https://doc.rust-lang.org/rustc/platform-support/apple-ios.html) (macOS required)
- [Android NDK](https://developer.android.com/ndk/downloads) and `ANDROID_NDK_HOME` environment variable

### Installation

```bash
# NPM
npm install --dev @craby/cli

# pnpm
pnpm install --dev @craby/cli

# Yarn
yarn add --dev @craby/cli
```

### Setup

TBD (Scaffold)

```bash
# Generates Rust code based on your TurboModule schemas.
craby codegen

# Compiles your Rust code into native binaries for Android and iOS.
craby build
```

## Commands

### `craby codegen`

Generates Rust/C++ code based on your TypeScript/Flow TurboModule schemas. This command:

- Analyzes your TurboModule spec files
- Generates corresponding Rust function signatures and C++ bridging implementations
- Generates native bindings for Android(CMakefile) and iOS(XCFramework)

### `craby build`

Compiles Rust code and generates native binaries.

### `craby show`

Displays project information and schemas.

### `craby doctor`

Checks project configuration and dependencies.

### `craby clean`

Cleans up temporary generated files.

## Project Structure

After running `craby init`, your project structure will look like this:

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
import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface Spec extends TurboModule {
  add(a: number, b: number): number;
  subtract(a: number, b: number): number;
}

export default TurboModuleRegistry.getEnforcing<Spec>('Calculator');
```

```typescript
// src/index.ts
import Calculator from './NativeCalculator';

export function add(a: number, b: number): number {
  return Calculator.add(a, b);
}

export function subtract(a: number, b: number): number {
  return Calculator.subtract(a, b);
}
```

After running `craby codegen`, you'll get:

```rust,ignore
// crates/lib/src/generated.rs (auto-generated)
use crate::ffi::calculator::*;

pub trait CalculatorSpec {
    fn add(a: f64, b: f64) -> f64;
    fn subtract(a: f64, b: f64) -> f64;
}
```

```rust,ignore
// crates/lib/src/calculator_impl.rs
use crate::{ffi::calculator::*, generated::*};

pub struct Calculator;

impl Calculator for CalculatorSpec {
    fn add(a: f64, b: f64) -> f64 {
        unimplemented!(); // Implement here!
    }
    
    fn subtract(a: f64, b: f64) -> f64 {
        unimplemented!(); // Implement here!
    }
}
```

### Android Setup

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

### iOS Setup

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
