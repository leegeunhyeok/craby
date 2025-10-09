# Sync vs Async

This guide explains the difference between synchronous and asynchronous methods in Craby, and when to use each approach.

## Overview

Craby supports two execution models:

1. **Synchronous (Sync)**: Methods that execute immediately on the JS thread and return values directly
2. **Asynchronous (Async)**: Methods that return `Promise` and execute in separate threads without blocking

## Synchronous Methods

Synchronous methods execute **immediately** on the **JavaScript thread** and return their result directly.

### Defining Sync Methods

```typescript
export interface Spec extends NativeModule {
  add(a: number, b: number): number;
  formatString(text: string): string;
  isValid(value: boolean): boolean;
}
```

### Implementation

```rust
impl CalculatorSpec for Calculator {
    fn add(&mut self, a: Number, b: Number) -> Number {
        a + b  // Returns immediately
    }

    fn format_string(&mut self, text: String) -> String {
        text.to_uppercase()  // Returns immediately
    }

    fn is_valid(&mut self, value: Boolean) -> Boolean {
        !value  // Returns immediately
    }
}
```

### JavaScript Usage

```typescript
// Executes immediately, blocks until complete
const result = Calculator.add(5, 3);
console.log(result); // 8

const formatted = Calculator.formatString("hello");
console.log(formatted); // "HELLO"
```

### When to Use Sync Methods

Use synchronous methods when:

<div class="tossface">

- ✅ Operation completes in **< 16ms** (one frame at 60fps)
- ✅ Simple calculations or data transformations
- ✅ No heavy computations
- ✅ Immediate result is needed

</div>

**Examples of good sync methods:**
- Math calculations
- String formatting
- Simple data validation
- Type conversions

## Asynchronous Methods

Asynchronous methods return `Promise<T>` and execute in **separate threads** (managed by C++ layer), keeping the UI responsive.

### Defining Async Methods

```typescript
export interface Spec extends NativeModule {
  calculatePrime(n: number): Promise<number>;
  sortLargeArray(numbers: number[]): Promise<number[]>;
  computeHash(data: string): Promise<string>;
}
```

### Implementation

```rust
impl HeavyComputeSpec for HeavyCompute {
    fn calculate_prime(&mut self, n: Number) -> Promise<Number> {
        if n <= 0.0 {
            return promise::reject("Invalid input");
        }

        // Long-running computation runs in separate thread
        let prime = nth_prime(n as i64);
        promise::resolve(prime as f64)
    }

    fn sort_large_array(&mut self, mut numbers: Array<Number>) -> Promise<Array<Number>> {
        // Heavy sorting operation - runs in separate thread
        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
        promise::resolve(numbers)
    }

    fn compute_hash(&mut self, data: String) -> Promise<String> {
        // CPU-intensive hashing - safe here in separate thread
        let hash = expensive_hash_algorithm(&data);
        promise::resolve(hash)
    }
}
```

### JavaScript Usage

```typescript
// Non-blocking - UI stays responsive
const prime = await HeavyCompute.calculatePrime(10000);
console.log('10000th prime:', prime);

// Or with .then()
HeavyCompute.sortLargeArray([5, 2, 9, 1, 7])
  .then(sorted => console.log('Sorted:', sorted))
  .catch(error => console.error('Error:', error));
```

### When to Use Async Methods

Use asynchronous methods when:

<div class="tossface">

- ✅ Operation takes **> 16ms** (would drop frames)
- ✅ CPU-intensive computations
- ✅ You want to keep the UI responsive
- ✅ Operation can fail and needs error handling

</div>

**Examples of good async methods:**
- Prime number calculations
- Large array sorting/filtering
- Cryptographic operations (hashing, encryption)
- Complex algorithms (graph traversal, pattern matching)
- Heavy data transformations

## Performance Comparison

### Sync Method Performance

```typescript
export interface Spec extends NativeModule {
  // blocks UI thread
  slowCalculation(): number;
}
```

```rust
impl BadExampleSpec for BadExample {
    fn slow_calculation(&mut self) -> Number {
        // This will FREEZE the UI for `expensive_job` durations!
        let res = expensive_job();
        res
    }
}
```

```typescript
console.log('Starting...');
const result = BadExample.slowCalculation();  // UI frozen
console.log('Done:', result);
```

### Async Method Performance

```typescript
export interface Spec extends NativeModule {
  // Runs in separate thread
  slowCalculation(): Promise<number>;
}
```

```rust
impl GoodExampleSpec for GoodExample {
    fn slow_calculation(&mut self) -> Promise<Number> {
        let res = expensive_job();
        promise::resolve(res)
    }
}
```

```typescript
console.log('Starting...');
const result = await GoodExample.slowCalculation();  // UI stays responsive!
console.log('Done:', result);
```

## Separate Thread Execution

When implementing Promise methods, your Rust code runs in a **separate thread** (spawned by C++ layer):

```rust
fn process_data(&mut self, value: Number) -> Promise<Number> {
    // This runs in a separate thread automatically
    // Safe to do heavy computations here
    let result = expensive_calculation(value);
    promise::resolve(result)
}

fn compute_fibonacci(&mut self, n: Number) -> Promise<Number> {
    // CPU-intensive recursive calculation
    let result = fibonacci(n as u64);
    promise::resolve(result as f64)
}
```

## Error Handling

### Sync Methods

Sync methods typically use **panics** for errors (which crash the app):

```rust
fn divide(&mut self, a: Number, b: Number) -> Number {
    if b == 0.0 {
        throw!("Division by zero");  // throw as JavaScript Error
    }
    a / b
}
```

```typescript
try {
  const result = Calculator.divide(123);
  console.log(result);
} catch (error) {
  console.error('Failed:', error);
}
```

### Async Methods

Async methods use **Promise rejections** for errors:

```rust
fn get_user(&mut self, id: Number) -> Promise<User> {
    if id <= 0.0 {
        return promise::reject("Invalid user ID");
    }

    // This runs in separate thread (managed by C++ layer)
    match database.find(id) {
        Some(user) => promise::resolve(user),
        None => promise::reject("User not found"),
    }
}
```

```typescript
try {
  const user = await UserService.getUser(123);
  console.log(user);
} catch (error) {
  console.error('Failed:', error);
}
```

## Summary

| Aspect | Sync | Async (Promise) |
|--------|------|-----------------|
| **Execution** | JS thread | Separate thread |
| **Returns** | `T` | `Promise<T>` |
| **Duration** | < 16ms | Any duration |
| **Heavy Work** | Avoid | Perfect for |
| **Error Handling** | Panic | Both Panic and Promise rejection |
| **UI Impact** | Blocking | Non-blocking |
| **Use Cases** | Math, formatting | Heavy compute, complex algorithms |
