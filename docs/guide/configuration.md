# Configuration

This guide covers Craby's configuration options.

## Config File

The `craby.toml` file in your project root defines both the code generation and platform-specific configurations:

```toml
[project]
name = "my_project"
source_dir = "src"

[android]
package_name = "rs.craby.calculator"
```

## Project Configuration

The `[project]` section configures code generation behavior:

- **`name`** (required): The name of your project. Used for naming generated modules, files, and build artifacts.
- **`source_dir`** (required): The directory path to scan for TypeScript source files. Craby will recursively search this directory to find spec files for code generation.

::: warning
Spec files **must** be prefixed with `Native` (e.g., `NativeCalculator.ts`) to be recognized by the code generator.
:::

## Android Configuration

The `[android]` section configures Android-specific settings:

- **`package_name`** (required): The Java package name for generated Kotlin/Android native module. Must follow reverse domain notation (e.g., `rs.craby.calculator`, `com.example.ㅡㅐ여ㅣㄷ`).
  - Format: Start with lowercase letter, can contain lowercase letters, numbers, underscores, and dots
  - Used in: AndroidManifest.xml, build.gradle namespace, Kotlin package declaration, and directory structure
