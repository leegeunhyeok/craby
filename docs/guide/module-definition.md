# Module Definition

This guide explains how to define your native module using TypeScript specs.

## Module Structure

::: info
Craby scans the source directory specified in your [configuration](/guide/configuration) for spec files prefixed with `Native` (e.g., `NativeCalculator.ts`). Only files matching this pattern will be processed by the code generator.
:::

Every Craby module starts with a TypeScript spec that extends `NativeModule` interface:

```typescript
// NativeMyModule.ts
import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface Spec extends NativeModule {
  add(a: number, b: number): number;
  greet(name: string): string;
}

export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
```

### Getting Module Instances

Craby provides two methods to get module instances:

```typescript
import { NativeModuleRegistry } from 'craby-modules';

NativeModuleRegistry.getEnforcing<Spec>('MyModule');
NativeModuleRegistry.get<Spec>('MyModule');
```

- `getEnforcing` - Returns the module instance. Throws an error if the module is not found (e.g., not linked).
- `get` - Returns the module instance if found, or `null` if the module doesn't exist.

## Defining Methods

Methods in your spec become Rust trait methods:

```typescript
export interface Spec extends NativeModule {
  // Synchronous method
  square(n: number): number;

  // Asynchronous method (returns Promise)
  calculatePrime(n: number): Promise<number>;

  // With user-defined types
  getSomething(): Something;
}
```

## Defining Types

You can define custom types using TypeScript interfaces:

```typescript
export interface Something {
  foo: string;
  bar: number;
  baz: string;
}
```

## Code Generation

When you run `crabygen` command, Craby generates Rust code from your TypeScript spec:

### Generated Rust Trait

```rust
// Auto-generated from TypeScript module spec
pub trait MyModuleSpec {
    fn square(&mut self, n: Number) -> Number;
    fn calculate_prime(&mut self, n: Number) -> Promise<User>;
    fn get_something(&mut self) -> Something;
}
```

### Generated Rust Structs

```rust
// Auto-generated from TypeScript interfaces
pub struct User {
    pub name: String,
    pub age: Number,
    pub email: String,
}
```

You just implement the generated trait!

```rust
#[craby_module]
impl MyModuleSpec for MyModule {
    fn square(&mut self, n: Number) -> Number {
        n * n
    }

    fn calculate_prime(&mut self, n: Number) -> Promise<Number> {
        let prime = nth_prime(n as i64);
        promise::resolve(prime as f64)
    }

    fn get_something(&mut self) -> Something {
        Something::default()
    }
}
```

## Required Module Methods

Every module must implement two required methods:
- `fn new(ctx: Context) -> Self` - Constructor that receives a Context
- `fn id(&self) -> usize` - Returns a unique identifier for the module instance

The `#[craby_module]` procedural macro automatically provides default implementations for these methods. However, you can override them if needed.

```rust
#[craby_module]
impl MyModuleSpec for MyModule {
    fn new(ctx: Context) -> Self {
        Self { ctx, count: 0 }
    }

    fn square(&mut self, n: Number) -> Number {
        n * n
    }
}
```

## Supported Types

Craby supports various TypeScript types. see the [Types](/guide/types) guide.
