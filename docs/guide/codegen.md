# Code Generation

This guide explains how Craby's code generation works and what files are generated.

## Overview

Craby analyzes your TypeScript NativeModule specs and automatically generates:

1. **Rust trait definitions** - Interface your implementation must follow
2. **Rust implementation templates** - Boilerplate for your module struct
3. **FFI layer** - Rust-to-C++ bridging code using cxx
4. **C++ bridge code** - Pure C++ TurboModule implementation
5. **Native build configs** - CMake for Android, XCFramework setup for iOS

## Running Code Generation

The primary command for code generation is:

```bash
npx crabygen codegen
```

Or simply:

```bash
npx crabygen
```

This command:
1. Scans your `src/` directory for TurboModule specs
2. Analyzes the TypeScript types
3. Generates corresponding Rust and C++ code
4. Updates build configurations

## Generated Files

### Rust Files

#### `crates/lib/src/generated.rs`

Contains the trait definition that your module must implement.

**Example input (TypeScript):**

```typescript
export interface Spec extends NativeModule {
  add(a: number, b: number): number;
  greet(name: string): string;
}
```

**Generated output (Rust):**

```rust
#[rustfmt::skip]
use crate::ffi::bridging::*;
use crate::types::*;

pub trait CalculatorSpec {
    fn new(id: usize) -> Self;
    fn id(&self) -> usize;
    fn add(&self, a: Number, b: Number) -> Number;
    fn greet(&self, name: String) -> String;
}
```

::: tip
This file is **auto-generated** and will be overwritten on each codegen run. Never edit this file manually!
:::

#### `crates/lib/src/module_impl.rs`

Your implementation file. This is where you write your actual Rust logic.

**Generated template:**

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

    fn add(&self, a: Number, b: Number) -> Number {
        unimplemented!() // TODO: Implement your logic here
    }

    fn greet(&self, name: String) -> String {
        unimplemented!() // TODO: Implement your logic here
    }
}
```

::: warning
This file is **generated only once**. After the initial generation, you can freely edit it—your changes will be preserved on subsequent codegen runs.
:::

#### `crates/lib/src/ffi.rs`

FFI (Foreign Function Interface) layer that bridges Rust and C++.

**Example:**

```rust
#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type Calculator;

        fn calculator_new(id: usize) -> Box<Calculator>;
        fn calculator_add(self: &Calculator, a: f64, b: f64) -> f64;
        fn calculator_greet(self: &Calculator, name: &str) -> String;
    }
}
```

::: tip
This file is **auto-generated**. It uses the [cxx](https://cxx.rs/) crate for safe Rust-C++ interop.
:::

#### `crates/lib/src/types.rs`

Helper types and type aliases used across your module.

```rust
pub type Number = f64;
pub type Boolean = bool;
pub type Void = ();
pub type Array<T> = Vec<T>;

// ... other helper types
```

#### `crates/lib/src/lib.rs`

Module entry point that ties everything together.

```rust
pub(crate) mod ffi;
pub(crate) mod generated;
pub(crate) mod types;

pub(crate) mod calculator_impl;
```

### C++ Files

#### `cpp/NativeCalculator.h` and `cpp/NativeCalculator.cpp`

Pure C++ TurboModule implementation that integrates with React Native.

**Header example:**

```cpp
#pragma once

#include <NativeCalculatorSpecJSI.h>

namespace facebook::react {

class NativeCalculator : public NativeCalculatorCxxSpec<NativeCalculator> {
public:
  NativeCalculator(std::shared_ptr<CallInvoker> jsInvoker);

  double add(jsi::Runtime& rt, double a, double b);
  std::string greet(jsi::Runtime& rt, std::string name);
};

} // namespace facebook::react
```

The C++ layer calls into your Rust code via the FFI boundary.

### Build Configuration Files

#### `android/CMakeLists.txt`

CMake configuration for building on Android:

```cmake
cmake_minimum_required(VERSION 3.13)
project(Calculator)

# Add Rust static library
add_library(calculator STATIC IMPORTED)
set_target_properties(calculator PROPERTIES
  IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../target/${ANDROID_ABI}/release/libcalculator.a
)

# Link everything together
target_link_libraries(
  ${CMAKE_PROJECT_NAME}
  calculator
  # ... other dependencies
)
```

#### iOS XCFramework

For iOS, Craby generates a fat binary (XCFramework) that includes all required architectures:

- `aarch64-apple-ios` - iOS devices (arm64)
- `aarch64-apple-ios-sim` - iOS Simulator on Apple Silicon

The XCFramework is placed in `ios/framework/libmodule.xcframework`.

## Incremental Code Generation

Craby's codegen is **incremental**:

- ✅ **Always regenerated**: `generated.rs`, `ffi.rs`, `types.rs`, C++ files
- ✅ **Generated once, preserved**: `*_impl.rs` implementation files
- ✅ **Never touched**: Your custom Rust code and business logic

This means you can:
1. Add new methods to your TypeScript spec
2. Run `crabygen codegen`
3. Implement the new methods in your `*_impl.rs` file

Your existing implementations remain untouched!

## Code Generation Options

You can customize code generation behavior:

### Watch Mode

Automatically regenerate code when specs change:

```bash
npx crabygen codegen --watch
```

### Specific Modules

Generate code for specific modules only:

```bash
npx crabygen codegen --module Calculator
```

### Verbose Output

See detailed generation logs:

```bash
npx crabygen codegen --verbose
```

## Understanding Generated Code

### Type Mapping

Here's how TypeScript types map to generated Rust code:

| TypeScript | Generated Rust |
|------------|----------------|
| `number` | `Number` (alias for `f64`) |
| `string` | `String` |
| `boolean` | `Boolean` (alias for `bool`) |
| `void` | `Void` (alias for `()`) |
| `T[]` | `Array<T>` (alias for `Vec<T>`) |
| `T \| null` | `Nullable<T>` |
| `Promise<T>` | `Promise<T>` |
| `{ a: number }` | `struct { pub a: Number }` |
| `enum E { A = 'a' }` | `enum E { A }` |

### Method Name Conversion

Method and field names are automatically converted:

| TypeScript | Rust |
|------------|------|
| `getUserName` | `get_user_name` |
| `isActive` | `is_active` |
| `phoneNumber` | `phone_number` |

### Signal Generation

Signals are special. For a TypeScript spec with signals:

```typescript
export interface Spec extends NativeModule {
  onData: Signal<{ value: number }>;
  onError: Signal<{ message: string }>;
}
```

Craby generates:

```rust
pub enum MyModuleSignal {
    OnData { value: Number },
    OnError { message: String },
}

// In your impl:
impl Spec for MyModule {
    fn some_method(&self) {
        self.emit(MyModuleSignal::OnData { value: 42.0 });
    }
}
```

## Validating Generated Code

After generation, it's a good idea to verify everything is correct:

### Check Rust Compilation

```bash
cargo check --manifest-path crates/lib/Cargo.toml
```

### Run Type Checking

```bash
npm run typecheck
# or: npx tsc --noEmit
```

### View Generated Specs

See what modules and methods were detected:

```bash
npx crabygen show
```

## Common Issues

### Module Not Found

If your module isn't being detected:

1. Ensure the file matches `Native*.ts` pattern
2. Check that it exports `NativeModule` interface
3. Verify it's in the `src/` directory

### Type Not Supported

If you get an "unsupported type" error:

1. Review [supported types](/guide/module-definition#supported-types)
2. Check for unsupported TypeScript features (unions, tuples, etc.)
3. Simplify complex types into smaller interfaces

### Generated Code Won't Compile

If generated Rust code has compilation errors:

1. Run `npx crabygen doctor` to check setup
2. Ensure Rust toolchain version matches `rust-toolchain.toml`
3. Check for naming conflicts with Rust keywords
4. Report the issue if it seems like a bug

## Next Steps

- [Building](/guide/building) - Learn how to build native binaries
- [Module Definition](/guide/module-definition) - Understand supported types
- [Examples](/examples) - See complete examples
