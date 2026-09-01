<div align="center">

<img width="360" src="./logo.svg" alt="logo">

# craby

Type-safe Rust for React Native—auto generated, integrated with pure C++ TurboModule

</div>

## Overview

Craby is a type-safe Rust development tool for React Native. It automatically generates Rust/C++ bindings from TypeScript schemas and integrates directly with **pure C++ TurboModule**, bypassing platform-specific layers like `ObjCTurboModule` and `JavaTurboModule`.

## Requirements

- React Native 0.76 or later with the New Architecture enabled
- Node.js 18 or later (React Native 0.87 requires Node.js 22.13 or later)
- Rust 1.90 or later
- macOS with Xcode 15.1 or later for iOS builds
- Android NDK for Android builds

## Getting Started

```bash
npx crabygen init <package-name>
```

Visit the [documentation](https://craby.rs) for installation, build, CocoaPods, and Swift Package Manager guides.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
