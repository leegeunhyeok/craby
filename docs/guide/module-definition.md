# Module Definition

This guide explains how to define your native module using TypeScript specs and how these types map to Rust.

## Basic Module Structure

Every Craby module starts with a TypeScript spec that extends `NativeModule`:

```typescript
import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface Spec extends NativeModule {
  // Your methods here
}

export default NativeModuleRegistry.getEnforcing<Spec>('YourModuleName');
```

## Supported Types

Craby supports a rich set of types that automatically map between TypeScript and Rust.

### Primitives

| TypeScript | Rust | Example |
|------------|------|---------|
| `number` | `Number` (f64) | Floating-point numbers |
| `string` | `String` | UTF-8 strings |
| `boolean` | `Boolean` (bool) | True/false values |
| `void` | `Void` (()) | No return value |

#### TypeScript

```typescript
export interface Spec extends NativeModule {
  addNumbers(a: number, b: number): number;
  greet(name: string): string;
  isValid(value: boolean): boolean;
  performAction(): void;
}
```

#### Generated Rust

```rust
pub trait Spec {
    fn add_numbers(&self, a: Number, b: Number) -> Number;
    fn greet(&self, name: String) -> String;
    fn is_valid(&self, value: Boolean) -> Boolean;
    fn perform_action(&self) -> Void;
}
```

### Objects

Complex objects are automatically converted between TypeScript interfaces and Rust structs.

#### TypeScript

```typescript
export interface User {
  name: string;
  age: number;
  isActive: boolean;
}

export interface Spec extends NativeModule {
  createUser(user: User): User;
}
```

#### Generated Rust

```rust
pub struct User {
    pub name: String,
    pub age: Number,
    pub is_active: Boolean,
}

pub trait Spec {
    fn create_user(&self, user: User) -> User;
}
```

#### Implementation Example

```rust
impl Spec for MyModule {
    fn create_user(&self, mut user: User) -> User {
        user.age += 1;
        user.name = format!("Welcome, {}!", user.name);
        user
    }
}
```

### Arrays

Arrays are represented using the `Array<T>` type in Rust.

#### TypeScript

```typescript
export interface Spec extends NativeModule {
  processNumbers(values: number[]): number[];
  joinStrings(values: string[]): string;
}
```

#### Generated Rust

```rust
pub trait Spec {
    fn process_numbers(&self, values: Array<Number>) -> Array<Number>;
    fn join_strings(&self, values: Array<String>) -> String;
}
```

#### Implementation Example

```rust
impl Spec for MyModule {
    fn process_numbers(&self, mut values: Array<Number>) -> Array<Number> {
        // Modify in place
        values.iter_mut().for_each(|x| *x *= 2.0);
        values
    }

    fn join_strings(&self, values: Array<String>) -> String {
        values.join(", ")
    }
}
```

### Enums

Craby supports both string and numeric enums.

#### String Enums

```typescript
export enum Status {
  Active = 'active',
  Inactive = 'inactive',
  Pending = 'pending',
}

export interface Spec extends NativeModule {
  processStatus(status: Status): string;
}
```

#### Numeric Enums

```typescript
export enum Priority {
  Low = 0,
  Medium = 1,
  High = 2,
}

export interface Spec extends NativeModule {
  processPriority(priority: Priority): number;
}
```

#### Implementation Example

```rust
impl Spec for MyModule {
    fn process_status(&self, status: Status) -> String {
        match status {
            Status::Active => "Currently active".to_string(),
            Status::Inactive => "Not active".to_string(),
            Status::Pending => "Waiting".to_string(),
            _ => unreachable!(),
        }
    }

    fn process_priority(&self, priority: Priority) -> Number {
        match priority {
            Priority::Low => 1.0,
            Priority::Medium => 5.0,
            Priority::High => 10.0,
            _ => unreachable!(),
        }
    }
}
```

### Nullable Types

Use union types with `null` to create optional values.

#### TypeScript

```typescript
export interface Spec extends NativeModule {
  processValue(value: number | null): number | null;
  formatName(name: string | null): string;
}
```

#### Generated Rust

```rust
pub trait Spec {
    fn process_value(&self, value: Nullable<Number>) -> Nullable<Number>;
    fn format_name(&self, name: Nullable<String>) -> String;
}
```

#### Implementation Example

```rust
impl Spec for MyModule {
    fn process_value(&self, value: Nullable<Number>) -> Nullable<Number> {
        match value.value_of() {
            Some(val) => {
                if *val < 0.0 {
                    Nullable::<Number>::none()  // Return null
                } else {
                    value.value(*val * 2.0)     // Return wrapped value
                }
            }
            None => Nullable::<Number>::some(0.0),  // Default value
        }
    }

    fn format_name(&self, name: Nullable<String>) -> String {
        match name.value_of() {
            Some(n) => format!("Hello, {}!", n),
            None => "Hello, Guest!".to_string(),
        }
    }
}
```

### Promises

Use `Promise<T>` for asynchronous operations.

#### TypeScript

```typescript
export interface Spec extends NativeModule {
  fetchData(id: number): Promise<string>;
  processAsync(value: number): Promise<number>;
}
```

#### Generated Rust

```rust
pub trait Spec {
    fn fetch_data(&self, id: Number) -> Promise<String>;
    fn process_async(&self, value: Number) -> Promise<Number>;
}
```

#### Implementation Example

```rust
use std::thread;
use std::time::Duration;

impl Spec for MyModule {
    fn fetch_data(&self, id: Number) -> Promise<String> {
        if id > 0.0 {
            // Promises execute asynchronously in a separate thread
            thread::spawn(move || {
                // Simulate heavy computation
                thread::sleep(Duration::from_secs(2));
                format!("Data for ID: {}", id)
            });
            promise::resolve(format!("Data for ID: {}", id))
        } else {
            promise::rejected("Invalid ID")
        }
    }

    fn process_async(&self, value: Number) -> Promise<Number> {
        // Heavy computations run in separate threads without blocking the JS thread
        thread::spawn(move || {
            // Perform expensive computation
            let result = complex_calculation(value);
            result
        });

        if value >= 0.0 {
            promise::resolve(value * 2.0)
        } else {
            promise::rejected("Value must be non-negative")
        }
    }
}
```

::: tip Async Execution
Promises in Craby execute **asynchronously** in **separate threads**, making them perfect for:
- Heavy computations that would block the UI
- File I/O operations
- Network requests
- Image/video processing
- Cryptographic operations

The JS thread remains responsive while your Rust code runs in the background!
:::

### Signals (Events)

Use `Signal` type to emit events from native to JavaScript. Signals are simple callbacks triggered from Rust without passing data.

#### TypeScript

```typescript
import type { NativeModule, Signal } from 'craby-modules';

export interface Spec extends NativeModule {
  onDataReceived: Signal;  // Signal name is the property name
  onError: Signal;
  onComplete: Signal;

  startListening(): void;
}
```

::: info Signal Names
The property name (e.g., `onDataReceived`) becomes the signal name. Signals do not carry data—they simply trigger callbacks in JavaScript.
:::

#### Generated Rust

```rust
pub trait Spec {
    fn start_listening(&self) -> Void;
}
```

#### Implementation Example

```rust
impl Spec for MyModule {
    fn start_listening(&self) -> Void {
        // Emit signals to trigger JavaScript callbacks
        self.emit(MyModuleSignal::OnDataReceived);

        // Do some work...
        process_data();

        // Emit completion signal
        self.emit(MyModuleSignal::OnComplete);

        // Or emit error signal if something fails
        if error_occurred {
            self.emit(MyModuleSignal::OnError);
        }
    }
}
```

#### JavaScript Usage

```typescript
import { MyModule } from 'your-module';

// Subscribe to signals (no data is passed)
MyModule.onDataReceived.addListener(() => {
  console.log('Data received signal triggered!');
  // Fetch the data separately if needed
});

MyModule.onError.addListener(() => {
  console.error('Error signal triggered!');
});

MyModule.onComplete.addListener(() => {
  console.log('Processing complete!');
});

// Start listening (triggers signals from native)
MyModule.startListening();
```

::: tip Use Cases for Signals
Signals are perfect for:
- Notifying JS when a background task completes
- Triggering UI updates from native events
- Simple event notifications without data payload
- Real-time event streams (sensors, timers, etc.)

If you need to pass data to JavaScript, consider using Promises or returning values from synchronous methods instead.
:::

## Nested Types

You can create complex nested structures:

```typescript
export interface Address {
  street: string;
  city: string;
  zipCode: string;
}

export interface User {
  name: string;
  age: number;
  address: Address | null;
  tags: string[];
}

export interface Spec extends NativeModule {
  updateUser(user: User): User;
}
```

This generates corresponding nested Rust structs:

```rust
pub struct Address {
    pub street: String,
    pub city: String,
    pub zip_code: String,
}

pub struct User {
    pub name: String,
    pub age: Number,
    pub address: Nullable<Address>,
    pub tags: Array<String>,
}
```

## Naming Conventions

Craby automatically converts between JavaScript and Rust naming conventions:

| TypeScript | Rust |
|------------|------|
| `camelCase` | `snake_case` |
| `myMethod` | `my_method` |
| `userName` | `user_name` |
| `isActive` | `is_active` |

## Best Practices

### 1. Keep Types Simple

Start with simple types and gradually add complexity:

```typescript
// ✅ Good: Start simple
export interface Spec extends NativeModule {
  add(a: number, b: number): number;
}

// ❌ Too complex initially
export interface Spec extends NativeModule {
  process(data: ComplexType[]): Promise<NestedResult | null>;
}
```

### 2. Use Type Aliases

Create type aliases for commonly used types:

```typescript
export type UserId = number;
export type Timestamp = number;

export interface Spec extends NativeModule {
  getUserData(id: UserId): Promise<UserData>;
}
```

### 3. Document Your Types

Add JSDoc comments to help users understand your API:

```typescript
export interface Spec extends NativeModule {
  /**
   * Calculates the sum of two numbers
   * @param a First number
   * @param b Second number
   * @returns The sum of a and b
   */
  add(a: number, b: number): number;
}
```

### 4. Avoid Unsupported Types

Some TypeScript types are not yet supported:

- ❌ Union types (except with `null`)
- ❌ Tuple types
- ❌ Function types
- ❌ Generic types (except Promise and Signal)

## Next Steps

- [Code Generation](/guide/codegen) - Learn how to generate Rust code
- [Building](/guide/building) - Build native binaries for your module
- [Examples](/examples) - See real-world examples
