# Getting Started

:::warning

This project is currently under development and is in early beta. For more information about the stable release, please refer to the following [link](https://github.com/leegeunhyeok/craby/issues/1)

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

## Create a Project

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

After installation, you'll need to set up the project structure manually (see [default template](https://github.com/leegeunhyeok/craby/tree/main/template)).

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
│           ├── ffi.rs            # Generated FFI layer
│           ├── types.rs          # Helper types
│           └── generated.rs      # Generated traits
├── cpp/                          # C++ implementations
├── android/                      # Android native setup
│   ├── build.gradle
│   └── CMakeLists.txt
├── ios/                          # iOS native setup
│   └── framework/                # Generated XCFramework
├── Cargo.toml                    # Root Cargo workspace
├── craby.toml                    # Craby config
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

When you run `crabygen` for the first time, it generates a default implementation file based on your module spec. Open `crates/lib/src/calculator_impl.rs` and implement the trait:

::: info

The default implementation file is only generated once to prevent overwriting your custom code. You can always reference the template in the `.craby` folder at your project root if needed.

:::

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

After building the native binaries, install CocoaPods dependencies for iOS:

```bash
cd ios && pod install
```

For Android, Gradle will automatically sync when you build the React Native app.

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
- [How to Build](/guide/build) - Build native binaries
