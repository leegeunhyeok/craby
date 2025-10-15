# Types

This guide explains the type system in Craby and how types are mapped between TypeScript, Rust, and C++.

## Type Mapping Overview

Craby automatically converts types between TypeScript, Rust, and C++ at compile-time using zero-cost abstractions.

| TypeScript | Rust | C++ |
|------------|------|-----|
| `boolean` | `bool` | `bool` |
| `number` | `f64` | `double` |
| `string` | `std::string::String` | `std::string` |
| `object` | `struct` | `struct` |
| `T[]` | `std::vec::Vec<T>` | `std::vector<T>` |
| `T \| null` | `Nullable<T>` | `struct` |
| `Promise<T>` | `std::result::Result<T, anyhow::Error>` | `T` (Unwrapped) |
| `enum` | `enum` | `enum class` |
| `void` | `()` | `void` |

::: info

- **Object types** are generated as structs matching your TypeScript schema
- **Nullable types** are generated using a pre-defined struct

:::

**Type Aliases**

Craby provides type aliases to make Rust types more familiar to TypeScript developers:

| Rust Type | Alias Type |
|-----------|------------|
| `bool` | `Boolean` |
| `f64` | `Number` |
| `std::string::String` | `String` |
| `std::vec::Vec<T>` | `Array<T>` |
| `std::result::Result<T, anyhow::Error>` | `Promise<T>` |
| `()` | `Void` |


## Number

Numbers in TypeScript map to `f64` (64-bit float) in Rust.

**TypeScript:**
```typescript
export interface Spec extends NativeModule {
  add(a: number, b: number): number;
}
```

**Rust:**
```rust
impl CalculatorSpec for Calculator {
    fn add(&mut self, a: Number, b: Number) -> Number {
        a + b
    }
}
```

## String

Strings are UTF-8 encoded and automatically converted between languages.

**TypeScript:**
```typescript
export interface Spec extends NativeModule {
  greet(name: string): string;
}
```

**Rust:**
```rust
impl GreeterSpec for Greeter {
    fn greet(&mut self, name: String) -> String {
        format!("Hello, {}!", name)
    }
}
```

## Boolean

**TypeScript:**
```typescript
export interface Spec extends NativeModule {
  isValid(value: boolean): boolean;
}
```

**Rust:**
```rust
impl ValidatorSpec for Validator {
    fn is_valid(&mut self, value: Boolean) -> Boolean {
        !value  // Negate the boolean
    }
}
```

## Objects

Custom objects are converted to Rust structs with public fields.

**TypeScript:**
```typescript
export interface User {
  name: string;
  age: number;
  email: string;
}

export interface Spec extends NativeModule {
  createUser(name: string, age: number, email: string): User;
}
```

**Generated Rust:**
```rust
pub struct User {
    pub name: String,
    pub age: Number,
    pub email: String,
}

impl UserManagerSpec for UserManager {
    fn create_user(&mut self, name: String, age: Number, email: String) -> User {
        User {
            name,
            age,
            email,
        }
    }
}
```

### Nested Objects

You can nest objects arbitrarily:

**TypeScript:**
```typescript
export interface Address {
  street: string;
  city: string;
}

export interface User {
  name: string;
  address: Address;
}
```

**Generated Rust:**
```rust
pub struct Address {
    pub street: String,
    pub city: String,
}

pub struct User {
    pub name: String,
    pub address: Address,
}
```

## Arrays

Arrays map to `std::vec::Vec<T>` in Rust and are wrapped in the `Array<T>` type.

**TypeScript:**
```typescript
export interface Spec extends NativeModule {
  sum(numbers: number[]): number;
  reverse(items: string[]): string[];
}
```

**Rust:**
```rust
impl ArrayProcessorSpec for ArrayProcessor {
    fn sum(&mut self, numbers: Array<Number>) -> Number {
        numbers.iter().sum()
    }

    fn reverse(&mut self, mut items: Array<String>) -> Array<String> {
        items.reverse();
        items
    }
}
```

### Working with Arrays

```rust
// Iterate over array
fn process(&mut self, items: Array<String>) -> Void {
    for item in items.iter() {
        println!("{}", item);
    }
}

// Modify in place
fn double(&mut self, mut numbers: Array<Number>) -> Array<Number> {
    numbers.iter_mut().for_each(|x| *x *= 2.0);
    numbers
}

// Create new array
fn generate(&mut self, count: Number) -> Array<Number> {
    (0..count as i32).map(|x| x as f64).collect()
}
```

## Nullable Types

Use `T | null` in TypeScript to create optional values.

**TypeScript:**
```typescript
export interface Spec extends NativeModule {
  findUser(id: number): User | null;
  formatName(name: string | null): string;
}
```

**Rust:**
```rust
impl UserServiceSpec for UserService {
    fn find_user(&mut self, id: Number) -> Nullable<User> {
        if id > 0.0 {
            Nullable::<User>::some(User { name: "John".to_string() })
        } else {
            Nullable::<User>::none()
        }
    }

    fn format_name(&mut self, name: Nullable<String>) -> String {
        match name.value_of() {
            Some(n) => format!("Hello, {}!", n),
            None => "Hello, Guest!".to_string(),
        }
    }
}
```

### Nullable API

```rust
// Create nullable values
let some_value = Nullable::<Number>::some(42.0);
let no_value = Nullable::<Number>::none();

// Check if value exists
if name.is_some() {
    // ...
}

// Get value as Option
match name.value_of() {
    Some(val) => println!("{}", val),
    None => println!("No value"),
}

// Unwrap (panics if None)
let value = name.unwrap();
```

## Enums

Craby supports both string and numeric enums.

### String Enums

**TypeScript:**
```typescript
export enum Status {
  Active = 'active',
  Inactive = 'inactive',
  Pending = 'pending',
}

export interface Spec extends NativeModule {
  getStatus(status: Status): string;
}
```

**Generated Rust:**
```rust
pub enum Status {
    Active,
    Inactive,
    Pending,
}

impl StatusCheckerSpec for StatusChecker {
    fn get_status(&mut self, status: Status) -> String {
        match status {
            Status::Active => "Currently active".to_string(),
            Status::Inactive => "Not active".to_string(),
            Status::Pending => "Waiting".to_string(),
            _ => unreachable!(),
        }
    }
}
```

### Numeric Enums

**TypeScript:**
```typescript
export enum Priority {
  Low = 0,
  Medium = 1,
  High = 2,
}

export interface Spec extends NativeModule {
  setPriority(priority: Priority): void;
}
```

**Generated Rust:**
```rust
pub enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
}

impl TaskManagerSpec for TaskManager {
    fn set_priority(&mut self, priority: Priority) -> Void {
        match priority {
            Priority::Low => println!("Low priority"),
            Priority::Medium => println!("Medium priority"),
            Priority::High => println!("High priority"),
            _ => unreachable!(),
        }
    }
}
```

## Promises

Promises enable asynchronous operations. When you return a Promise, the C++ layer automatically executes your Rust code in a separate thread.

**TypeScript:**
```typescript
export interface Spec extends NativeModule {
  processAsync(value: number): Promise<number>;
}
```

**Rust:**
```rust
impl AsyncServiceSpec for AsyncService {
    fn process_async(&mut self, value: Number) -> Promise<Number> {
        // Runs in separate thread (managed by C++ layer)
        // Safe to do heavy work here
        if value >= 0.0 {
            promise::resolve(value * 2.0)
        } else {
            promise::reject("Negative values not allowed")
        }
    }
}
```

See [Sync vs Async](/guide/sync-vs-async) for more details on async operations.

## Limitations

Craby supports fewer types than standard TurboModule to maintain simplicity and focus on performance-critical use cases. Types not listed in the supported types table are not available.
