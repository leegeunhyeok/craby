# How to build

This guide covers building native binaries for iOS and Android using Craby.

## Overview

After implementing your module in Rust, you need to compile it into native binaries that can be used by React Native. Craby handles the entire build process for you.

## Building for All Platforms

The simplest way to build for all supported platforms:

```bash
npx craby build
```

This command:
1. Compiles Rust code for all target architectures
2. Generates platform-specific binaries
3. Packages them for use in React Native

By default, this builds for:
- **iOS**: `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios`
- **Android**: `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android`

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

Modify the build script according to your package manager and build tools.

## Build Targets

### iOS Architectures

| Architecture | Target | Description |
|-------------|--------|-------------|
| Device (arm64) | `aarch64-apple-ios` | Physical iOS devices (iPhone, iPad) |
| Simulator (arm64) | `aarch64-apple-ios-sim` | iOS Simulator on Apple Silicon Macs |
| Simulator (x86_64) | `x86_64-apple-ios` | iOS Simulator on Intel Macs |

### Android Architectures

| Architecture | Target | Description |
|-------------|--------|-------------|
| arm64-v8a | `aarch64-linux-android` | 64-bit ARM devices (most modern phones) |
| armeabi-v7a | `armv7-linux-androideabi` | 32-bit ARM devices (older phones) |
| x86_64 | `x86_64-linux-android` | 64-bit x86 emulator |
| x86 | `i686-linux-android` | 32-bit x86 emulator |

## Build Output

After a successful build:

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
- `target/` directory (Rust build artifacts)
- Generated binaries in `android/` and `ios/`
- Build caches
