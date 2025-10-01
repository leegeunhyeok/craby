# Getting Started

This guide will walk you through creating your first Craby module from scratch.

## Prerequisites

::: warning macOS Required
Craby development requires **macOS** with **Xcode 12 or higher** for building both iOS and Android targets.
:::

Before you begin, make sure you have the following installed:

### Prerequisites

- **macOS**: Required for building iOS targets
- **Xcode 12 or higher**: [Download](https://developer.apple.com/xcode)
- **Node.js 18+**: [Download](https://nodejs.org/)
- **Rust nightly-2025-08-04**: Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup toolchain install nightly-2025-08-04
  ```
- **Android NDK**: [Download](https://developer.android.com/ndk/downloads)
- **ANDROID_NDK_HOME environment variable**: Set to your NDK path
  ```bash
  export ANDROID_NDK_HOME=/path/to/android-ndk
  ```

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

This generates:
- `crates/lib/src/generated.rs` - Rust trait definition
- `crates/lib/src/calculator_impl.rs` - Implementation template
- `crates/lib/src/ffi.rs` - FFI bridging code
- C++ bridge files in `cpp/`
- Native build configurations

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

    fn add(&self, a: Number, b: Number) -> Number {
        a + b
    }

    fn subtract(&self, a: Number, b: Number) -> Number {
        a - b
    }

    fn multiply(&self, a: Number, b: Number) -> Number {
        a * b
    }

    fn divide(&self, a: Number, b: Number) -> Number {
        a / b
    }
}
```

### Step 4: Build Native Binaries

Build the Rust code for all target platforms:

```bash
npx crabygen build
```

This compiles your Rust code and generates:
- **Android**: Static libraries in `android/src/main/jni/libs`
- **iOS**: XCFramework in `ios/framework/`

### Step 5: Use in Your React Native App

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

- [Module Definition](/guide/module-definition) - Learn about supported types and patterns
- [Code Generation](/guide/codegen) - Understand the code generation process
- [Building](/guide/building) - Deep dive into the build system
- [Advanced Features](/guide/advanced) - Promises, signals, and complex types
