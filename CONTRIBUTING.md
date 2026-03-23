# Contributing to Craby

Thank you for your interest in contributing to Craby!

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Help](#getting-help)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Unit Tests](#unit-tests)
- [E2E Testing](#e2e-testing)
- [Code Quality Checks](#code-quality-checks)
- [Pull Request Process](#pull-request-process)
- [Commit Message Guidelines](#commit-message-guidelines)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [dev.ghlee@gmail.com](mailto:dev.ghlee@gmail.com).

## Getting Help

If you have questions or need help getting started, use [GitHub Discussions](https://github.com/leegeunhyeok/craby/discussions).

## How to Contribute

- **Report bugs**: [Open an issue](https://github.com/leegeunhyeok/craby/issues/new) with a clear description and reproduction steps.
- **Suggest features**: Start a discussion or open an issue.
- **Improve documentation**: Fix typos, add examples, or clarify instructions.
- **Submit pull requests**: Fix bugs, add features, or improve existing code.

## Development Setup

Craby uses [mise](https://mise.jdx.dev/) to manage Node.js and Rust versions.

```bash
mise trust && mise install   # Rust nightly + Node LTS
yarn install                 # Install JS dependencies
yarn prepare                 # Full build (all packages)
```

### Build CLI bindings and run commands

```bash
yarn workspace @craby/cli-bindings build
yarn workspace crabygen run execute <command> [options]
```

### Test with the example module

```bash
cd examples/craby-test
yarn crabygen <command> [options]
```

## Unit Tests

### Rust

```bash
cargo test --all                       # Run all Rust unit tests
cargo insta test --workspace           # Run snapshot tests and collect diffs
cargo insta review --workspace         # Review and accept snapshot changes
```

> If your changes affect code generation, snapshot tests will fail. Always run `cargo insta review --workspace` and carefully inspect each diff before accepting.

### TypeScript

```bash
yarn workspaces foreach --all --topological-dev run typecheck   # Type check all packages
```

### NAPI bindings (Vitest)

The NAPI bindings tests load the native binary against an actual Metro bundle.
**Start the Metro bundler first**, then run the tests:

```bash
# Terminal 1 — start Metro
cd examples/craby-test && yarn crabygen start

# Terminal 2 — run Vitest
yarn workspace @craby/cli-bindings test
```

### Codegen changes checklist

When modifying `craby_codegen`:

1. Add the new type/method case to `crates/craby_codegen/src/tests/mod.rs`
2. Run `cargo insta test --workspace` → `cargo insta review --workspace` to update snapshots
3. Add the corresponding method to `examples/craby-test/src/NativeCrabyTest.ts` (TS spec) and `examples/craby-test/crates/lib/src/craby_test_impl.rs` (Rust impl), then regenerate:
   ```bash
   cd examples/craby-test && yarn crabygen codegen
   ```
4. Add an E2E assertion in `examples/test-suites/src/test-suites.ts`

## E2E Testing

E2E testing runs the full workflow on a real React Native app.

### Prerequisites

1. Build CLI bindings:
   ```bash
   yarn workspace @craby/cli-bindings build
   ```

2. Generate code and build native libraries:
   ```bash
   cd examples/craby-test
   yarn crabygen codegen
   yarn crabygen build
   ```

### Running with sample apps

Test against both React Native versions:

- `examples/0.80` — React Native 0.80
- `examples/0.76` — React Native 0.76

For each app:

#### 1. Start Metro bundler

Metro must be running before launching the app. Without it the JS bundle won't load and all tests will fail.

```bash
cd examples/<version>
yarn start
```

#### 2. iOS

```bash
yarn pod:install   # Install CocoaPods dependencies
yarn ios           # Build and launch on simulator
```

Or build manually via Xcode using `examples/<version>/ios/*.xcworkspace`.

#### 3. Android

```bash
yarn android       # Build and launch on emulator/device
```

Or build manually via Android Studio.

#### 4. Run tests

- Launch the app on your device/simulator/emulator
- Tap **Run All Tests**
- Verify all items show **Passed**

### Important notes

- Test on **both** React Native versions (0.76 and 0.80)
- Test on **both** iOS and Android
- All tests must pass before submitting a PR

## Code Quality Checks

Run these locally before opening a PR.

### TypeScript (Biome)

```bash
yarn lint:all    # Lint + format check
yarn lint:fix    # Auto-fix
```

### Rust

```bash
cargo clippy --all -- --deny warnings   # Lint
cargo fmt --all -- --check              # Format check
cargo fmt --all                         # Format (apply)
cargo test --all                        # Tests
```

## Pull Request Process

1. Fork the repository
2. Implement your change and run all quality checks
3. Commit following the [commit message guidelines](#commit-message-guidelines)
4. Open a PR against the `main` branch
5. A maintainer will approve the CI workflow — all checks must pass before merge

## Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]

[optional footer]
```

Common types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`

---

Thank you for contributing to Craby!
