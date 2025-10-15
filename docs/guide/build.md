# How to build

This guide covers building native binaries for iOS and Android using Craby.

## Overview

After implementing your module in Rust, you need to compile it into native binaries that can be used by React Native. Craby handles the entire build process for you.

## Building for All Platforms

The simplest way to build for all supported platforms:

::: info

Before building, Craby verifies that the generated code matches the current module specifications.

If they don't match, you won't be able to build and you'll need to run the `codegen` command to update the generated code.

:::

```bash
npx craby build
```

- Compiles Rust code for all target architectures
- Generates platform-specific binaries

---

By default, this builds for:

**Android**

| Identifier | Target | Description |
|-------------|--------|-------------|
| ios-arm64 | `aarch64-apple-ios` | Physical iOS devices (iPhone, iPad) |
| ios-arm64-simulator | `aarch64-apple-ios-sim` | iOS Simulator on Apple Silicon Macs |
| ios-x86_64-simulator | `x86_64-apple-ios` | iOS Simulator on Intel Macs |


::: info

The simulator target libraries (`ios-arm64-simulator` and `ios-x86_64-simulator`) are merged into a single universal binary (`ios-arm64_x86_64-simulator`) using `lipo` command line tool.

:::

**iOS**

| ABI | Target | Description |
|-------------|--------|-------------|
| arm64-v8a | `aarch64-linux-android` | 64-bit ARM devices (most modern phones) |
| armeabi-v7a | `armv7-linux-androideabi` | 32-bit ARM devices (older phones) |
| x86_64 | `x86_64-linux-android` | 64-bit x86 emulator |
| x86 | `i686-linux-android` | 32-bit x86 emulator |

## Setup scripts for publishing Craby modules

To integrate with package publishing, we recommend the following configuration:

```json
{
  "name": "my-module",
  "scripts": {
    "prepack": "npm build",
    "build": "craby build && tsdown",
  }
}
```

> Modify the build script according to your package manager and build tools.

## Build Output

### iOS

```
ios/framework/libmodule.xcframework/
├── Info.plist
├── ios-arm64/
│   └── libmodule-prebuilt.a
├── ios-arm64-simulator/
│   └── libmodule-prebuilt.a
└── ios-x86_64-simulator/
    └── libmodule-prebuilt.a
```

### Android

```
android/src/main/libs/
├── arm64-v8a/
│   └── libmodule-prebuilt.a
├── armeabi-v7a/
│   └── libmodule-prebuilt.a
├── x86/
│   └── libmodule-prebuilt.a
└── x86_64/
    └── libmodule-prebuilt.a
```

## Cleaning Build Artifacts

Remove all build artifacts and caches:

```bash
npx crabygen clean
```

This removes:

- `target` directory (Rust build artifacts)
- Generated binaries in `android` and `ios`
- Build caches
