# Contributing to Craby

Thank you for your interest in contributing to Craby!

We welcome contributions from the community and appreciate your efforts to help improve this project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Help](#getting-help)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Testing the CLI](#testing-the-cli)
- [E2E Testing](#e2e-testing)
- [Code Quality Checks](#code-quality-checks)
- [Pull Request Process](#pull-request-process)
- [Commit Message Guidelines](#commit-message-guidelines)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [dev.ghlee@gmail.com](mailto:dev.ghlee@gmail.com).

## Getting Help

If you have questions about the project or need help getting started, please use our [GitHub Discussions](https://github.com/leegeunhyeok/craby/discussions) page. This is the best place to ask questions, share ideas, and engage with the community.

## How to Contribute

There are many ways to contribute to Craby:

- **Report bugs**: If you find a bug, please [open an issue](https://github.com/leegeunhyeok/craby/issues/new) with a clear description and reproduction steps.
- **Suggest features**: Have an idea for a new feature? Start a discussion or open an issue to share your thoughts.
- **Improve documentation**: Help us improve our docs by fixing typos, adding examples, or clarifying instructions.
- **Submit pull requests**: Fix bugs, add features, or improve existing code.

## Development Setup

Craby uses [mise](https://mise.jdx.dev/) to manage Node.js and Rust versions, ensuring consistent development environments across the team.

1. **Set up the project**:
   ```bash
   mise trust
   mise install
   ```

2. **Install dependencies**:
   ```bash
   yarn install
   ```

3. **Build all packages**:
   ```bash
   yarn prepare
   ```

## Testing the CLI

### Building the Native Bindings

Before testing CLI commands, you need to build the NAPI-RS bindings:

```bash
yarn workspace @craby/cli-bindings build
```

### Running CLI Commands

After building the bindings, you can execute commands using:

```bash
yarn workspace crabygen run execute <command> [options]
```

### Testing Core Features

To test critical features like code generation and builds, use the test project:

```bash
cd examples/craby-test
yarn crabygen <command> [options]
```

This provides a real-world environment for testing your changes.

## E2E Testing

End-to-end testing ensures the entire workflow functions correctly across different React Native versions and platforms.

### Prerequisites

1. **Build the CLI bindings**:
   ```bash
   yarn workspace @craby/cli-bindings build
   ```

2. **Generate code and verify build**:
   ```bash
   cd examples/craby-test
   yarn crabygen
   yarn build
   ```

   If the build succeeds, proceed to test with sample apps.

### Testing with Sample Apps

Test your changes with the provided React Native sample apps to ensure compatibility:

- `examples/0.80` - React Native 0.80
- `examples/0.76` - React Native 0.76

For each sample app, follow these steps:

#### Start Metro Development Server

```bash
yarn start
```

#### Android Testing

1. **Build the app**:
   ```bash
   yarn android
   ```

   Alternatively, build manually using Android Studio.

2. **Run tests**:
   - Launch the app on your device/emulator
   - Tap the "Run All Tests" button
   - Verify that all test items pass

#### iOS Testing

1. **Install CocoaPods dependencies**:
   ```bash
   yarn pod:install
   ```

   This adds the built binary to the iOS workspace.

2. **Build the app**:
   ```bash
   yarn ios
   ```

   Alternatively, build manually using Xcode.

3. **Run tests**:
   - Launch the app on your device/simulator
   - Tap the "Run All Tests" button
   - Verify that all test items pass

### Important Notes

- Run E2E tests for **both** React Native versions (0.76 and 0.80)
- Test on **both** Android and iOS platforms
- Ensure all tests pass before submitting your PR
- If tests fail, investigate and fix the issues before proceeding

## Code Quality Checks

Before submitting a pull request, ensure your code passes all quality checks. Run these commands locally to catch issues early.

### TypeScript

- **Lint check**:
  ```bash
  yarn lint:all
  ```

- **Lint and auto-fix**:
  ```bash
  yarn lint:fix
  ```

- **Type checking**:
  ```bash
  yarn workspaces foreach --all --topological-dev run typecheck
  ```

### Rust

- **Clippy**:
  ```bash
  cargo clippy --all -- --deny warnings
  ```

- **Run tests**:
  ```bash
  cargo test --all
  ```

- **Review snapshot changes**:

  If your changes affect code generation, snapshot tests may fail. Review and accept changes with:
  ```bash
  cargo insta review --workspace
  ```

  Carefully review each snapshot change, and press `a` to accept valid changes.

## Pull Request Process

1. **Fork repository**

2. **Make your changes**: Implement your bug fix, feature, or improvement.

3. **Test locally**: Run all code quality checks mentioned above to ensure everything passes.

4. **Commit your changes**: Follow our [commit message guidelines](#commit-message-guidelines).

5. **Push and open a pull request**: Push your branch and open a PR against the `main` branch.

6. **CI approval**: After you open a PR, a maintainer will approve the CI workflow to run automated tests.

7. **CI validation**: The CI workflow will run all quality checks, build processes, and end-to-end tests. All checks must pass before your PR can be merged.

### Important Notes

- Make sure to run all quality checks locally before opening a PR. This speeds up the review process and reduces CI failures.
- The CI runs comprehensive validation including builds and E2E tests that verify the entire system works correctly.
- Address all feedback from maintainers promptly and update your PR as needed.

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. This helps us maintain a clear and consistent project history.

### Format

```
<type>: <description>

[optional body]

[optional footer]
```

---

Thank you for contributing to Craby! Your efforts help make this project better for everyone.
