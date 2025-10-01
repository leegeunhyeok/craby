# CLI Commands

This guide covers the Craby CLI commands for project initialization, code generation, and building.

## Overview

Craby provides a CLI tool called `crabygen` for managing your native modules:

```bash
npx crabygen <command> [options]
```

::: info
The `craby` command is an alias for `crabygen`
:::

## Commands

### `init`

Initialize a new Craby module project with complete scaffolding.

```bash
npx crabygen init <module-name>
```

**Arguments:**
- `<module-name>` - Name of your module (e.g., `my-calculator`)

**What it creates:**
- Project structure with TypeScript, Rust, and native build configurations
- Package configuration files
- Build scripts

**Example:**

```bash
npx crabygen init my-calculator
cd my-calculator
```

### `codegen`

Generates Rust and C++ bridge code from your TypeScript specs.

```bash
npx crabygen
# or
npx crabygen codegen
```

**What it does:**
- Analyzes your TypeScript module specs
- Generates Rust trait, struct definitions
- Generates C++ bridge code
- Generates FFI layer code
- Updates build configurations

**When to run:**
- After modifying TypeScript module specs
- After changing module name

**Generated files:**

```
crates/lib/src/
├── generated.rs        # Rust trait definitions
├── ffi.rs              # FFI bridging code
└── types.rs            # Helper types

cpp/
├── Cxx<Module>Module.cpp  # C++ TurboModule implementation
└── Cxx<Module>Module.h    # C++ headers
```

### `build`

Build native binaries for iOS and Android platforms.

```bash
npx crabygen build
```

**What it does:**
- Compiles Rust code for all target architectures
- Generates platform-specific binaries

**Target platforms:**
- **iOS**: `aarch64-apple-ios`, `aarch64-apple-ios-sim`
- **Android**: `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android`

**Build output:**

```
ios/framework/
└── lib<module>.xcframework/

android/src/main/libs/
├── arm64-v8a/
├── armeabi-v7a/
├── x86/
└── x86_64/
```

### `clean`

Remove all build artifacts and caches.

```bash
npx crabygen clean
```

**What it removes:**
- `target/` directory (Rust build artifacts)
- Generated binaries in `android/` and `ios/`
- Build caches

**When to use:**
- Before a fresh build
- When troubleshooting build issues
- When switching between branches
