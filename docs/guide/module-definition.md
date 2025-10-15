# Module Definition

This guide explains how to define your native module using TypeScript specs.

## Basic Module Structure

::: info

Module spec files must start with the "Native" prefix.

:::

Every Craby module starts with a TypeScript spec that extends `NativeModule`:

```typescript
// NativeMyModule.ts
import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface Spec extends NativeModule {
  // Your methods here
  add(a: number, b: number): number;
  greet(name: string): string;
}

export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
```

### Module Registration

- `NativeModule` - Base interface for all Craby modules
- `NativeModuleRegistry.getEnforcing<Spec>()` - Get your module instance

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
export interface User {
  name: string;
  age: number;
  email: string;
}
```

## Type Aliases

Use type aliases for better code organization:

```typescript
export type UserId = number;
export type Timestamp = number;

export interface User {
  id: UserId;
  createdAt: Timestamp;
}
```

## Code Generation

When you run `crabygen` command, Craby generates Rust code from your TypeScript spec:

### Generated Rust Trait

```rust
// Auto-generated from TypeScript spec
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

### Your Implementation

You implement the generated trait:

```rust
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

## Stateful Modules

Modules can maintain state across method calls. Each module instance preserves its internal Rust state between invocations:

```rust
struct Storage {
    id: usize,
    data: Option<Number>,
}

impl StorageSpec for Storage {
    fn set_data(&mut self, data: Number) -> Void {
        self.data = Some(data);
    }

    fn get_data(&mut self) -> Number {
        self.data.unwrap_or(0.0)
    }
}
```

```typescript
Storage.setData(123);
Storage.getData(); // 123
```

## Supported Types

Craby supports various TypeScript types. see the [Types](/guide/types) guide.
