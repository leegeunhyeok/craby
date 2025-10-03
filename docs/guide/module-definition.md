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

  // Method with no return value
  noop(): void;
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

export interface Spec extends NativeModule {
  createUser(name: string, age: number, email: string): User;
  updateUser(user: User): User;
}
```

### Type Aliases

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

When you run `crabygen`, Craby generates Rust code from your TypeScript spec:

### Generated Rust Trait

```rust
// Auto-generated from TypeScript spec
pub trait MyModuleSpec {
    fn square(&self, n: Number) -> Number;
    fn calculate_prime(&self, n: Number) -> Promise<Number>;
    fn noop(&self) -> Void;
    fn create_user(&self, name: String, age: Number, email: String) -> User;
    fn update_user(&self, user: User) -> User;
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
    fn square(&self, n: Number) -> Number {
        n * n
    }

    fn calculate_prime(&self, n: Number) -> Promise<Number> {
        let prime = nth_prime(n as i64);
        promise::resolve(prime as f64)
    }

    fn noop(&self) -> Void {
        ()
    }

    fn create_user(&self, name: String, age: Number, email: String) -> User {
        User { name, age, email }
    }

    fn update_user(&self, mut user: User) -> User {
        user.name = user.name.to_uppercase();
        user
    }
}
```

## Naming Conventions

Method and field names are automatically converted as `snake_case`:

```typescript
// TypeScript
interface Profile {
  name: string;
  homeAddress: string;
}

export interface Spec extends NativeModule {
  setUser(userId: number, profile: Profile): boolean;
}
```

```rust
// Generated Rust
struct Profile {
    name: String,
    home_address: String,
}

pub trait MyModuleSpec {
    fn get_user_name(&self, user_id: Number, profile: Profile) -> bool;
}
```

## Supported Types

Craby supports various TypeScript types. See the [Types](/guide/types) guide for detailed information:

- **Primitives**: `number`, `string`, `boolean`, `void`
- **Objects**: Custom interfaces
- **Arrays**: `T[]`
- **Enums**: String and numeric enums
- **Nullable**: `T | null`
- **Promises**: `Promise<T>`
- **Signals**: `Signal`

## Limitations

### Unsupported Types

Some TypeScript types are not supported:

<div class="tossface">

- ❌ Union types (except `T | null`)
- ❌ Tuple types
- ❌ Function types
- ❌ Generic types (except `Promise` and `Signal`)

</div>

### Stateless Modules

Craby modules are **stateless** - you cannot store and maintain state between method calls.

```rust
// Not supported - stateful module
struct Counter {
    count: i32,  // Cannot maintain state
}

impl CounterSpec for Counter {
    fn increment(&self) -> Number {
        self.count += 1;  // Won't work! (It will always be the initial value)
        self.count as f64
    }
}

// Supported - stateless operations
impl CalculatorSpec for Calculator {
    fn add(&self, a: Number, b: Number) -> Number {
        a + b  // Pure function, no state
    }
}
```

If you need to maintain state, manage it on the JavaScript side:

```typescript
// Manage state in JavaScript
let count = 0;

function increment() {
  count++;
  return Calculator.square(count);
}
```
