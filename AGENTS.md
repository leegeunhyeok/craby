# Craby — Agent Guide

Craby is a type-safe Rust development toolkit for React Native. It generates Rust/C++ bindings from TypeScript module specs and integrates them as pure C++ TurboModules, bypassing ObjC/Java bridge layers entirely.

## How It Works

1. **TypeScript spec** → OXC parses the spec into an intermediate Schema/IR
2. **Code generation** → Schema is transformed into platform-specific Rust, C++, iOS, and Android code
3. **FFI bridge** → the `cxx` crate provides Rust↔C++ interop
4. **RN integration** → generated code plugs in as a React Native pure C++ TurboModule

## Project Structure

```
craby/
├── crates/                  # Rust workspace
│   ├── craby/               # Core prelude (types, context, macros)
│   ├── craby_macro/         # #[craby_module] proc macro
│   ├── craby_common/        # Config (craby.toml), platform utils, logging
│   ├── craby_codegen/       # TS spec parser (OXC) + code generators (Rust/C++/iOS/Android)
│   ├── craby_build/         # Cargo build orchestration + XCFramework/AAR packaging
│   └── craby_cli/           # CLI commands (init/codegen/build/clean/doctor/show)
│
├── packages/                # Node packages
│   ├── cli-bindings/        # NAPI-RS bindings — exposes Rust CLI to Node.js
│   ├── crabygen/            # TypeScript CLI frontend (commander.js)
│   ├── craby-devkit/        # Metro config helpers, RN config utilities
│   └── craby-modules/       # NativeModuleRegistry, Signal type
│
├── examples/
│   ├── craby-test/          # Integration test module (used by E2E apps)
│   ├── test-suites/         # Shared RN test suite component
│   ├── 0.80/                # E2E app — React Native 0.80
│   └── 0.76/                # E2E app — React Native 0.76
│
└── docs/                    # Documentation site (Next.js / Fumadocs)
```

## Key Conventions

- **Rust nightly** — toolchain pinned in `rust-toolchain.toml`
- **Yarn Berry** — `node-modules` linker (PnP disabled for RN compat)
- **Biome** — TypeScript lint/format
- **Clippy + rustfmt** — Rust lint/format
- **insta** — snapshot testing for all codegen output (`cargo insta review --workspace` after changes)
- **Conventional Commits** — `feat:`, `fix:`, `chore:`, etc.

## Codegen Notes

`craby_codegen` is the most critical crate. When modifying code generation logic:

1. Edit `crates/craby_codegen/src/platform/cxx.rs` or `rust.rs` for type mapping / template changes
2. Add the new case to the test schema in `crates/craby_codegen/src/tests/mod.rs`
3. Run `cargo insta test --workspace` then `cargo insta review --workspace` to update snapshots
4. Add the corresponding method to `examples/craby-test/` (TS spec + Rust impl) and regenerate with `yarn crabygen codegen`
5. Add an E2E assertion in `examples/test-suites/src/test-suites.ts`

## Contribution Guidelines

See [CONTRIBUTING.md](CONTRIBUTING.md) for full setup instructions, quality checks, testing workflow, PR process, and commit conventions.
