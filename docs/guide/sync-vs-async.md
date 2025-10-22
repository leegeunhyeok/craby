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
}
```

### Implementation

```rust
impl LightComputeSpec for LightCompute {
    fn add(&mut self, a: Number, b: Number) -> Number {
        a + b  // Returns immediately
    }

    fn format_string(&mut self, text: &str) -> String {
        text.to_uppercase()  // Returns immediately
    }
}
```

### JavaScript Usage

```typescript
// Executes immediately, blocks until complete
const result = LightCompute.add(5, 3);
console.log(result); // 8

const formatted = LightCompute.formatString("hello");
console.log(formatted); // "HELLO"
```

### When to Use Sync Methods

Use synchronous methods when:


- <span class="tossface">✅</span> Operation completes in **< 16ms** (one frame at 60fps)
- <span class="tossface">✅</span> Simple calculations or data transformations
- <span class="tossface">✅</span> No heavy computations
- <span class="tossface">✅</span> Immediate result is needed


**Examples of good sync methods:**

- Basic math calculations
- String formatting
- Simple data validation
- Type conversions

## Asynchronous Methods

Asynchronous methods return `Promise<T>` and execute in **separate threads** (managed by C++ layer), keeping the UI responsive.

### Defining Async Methods

```typescript
export interface Spec extends NativeModule {
  calculatePrime(n: number): Promise<number>;
  computeHash(data: string): Promise<string>;
}
```

### Implementation

```rust
impl HeavyComputeSpec for HeavyCompute {
    fn calculate_prime(&mut self, n: Number) -> Promise<Number> {
        if n <= 0.0 {
            // Use the `reject` function from the `promise` module to reject the Promise
            return promise::reject("Invalid input");
        }

        // Long-running computation runs in separate thread
        let prime = nth_prime(n as i64);

        // Use the `resolve` function from the `promise` module to resolve the Promise
        promise::resolve(prime as f64)
    }

    fn compute_hash(&mut self, data: &str) -> Promise<String> {
        // CPU-intensive hashing - safe here in separate thread
        let hash = expensive_hash_algorithm(data);
        promise::resolve(hash)
    }
}
```

### JavaScript Usage

```typescript
// Non-blocking - UI stays responsive
const prime = await HeavyCompute.calculatePrime(10000);
console.log('10000th prime:', prime);

// Or with promise chaining
HeavyCompute.sortLargeArray([5, 2, 9, 1, 7])
  .then(sorted => console.log('Sorted:', sorted))
  .catch(error => console.error('Error:', error));
```

### When to Use Async Methods

Use asynchronous methods when:

- <span class="tossface">✅</span> Operation takes **> 16ms** (would drop frames)
- <span class="tossface">✅</span> CPU-intensive computations
- <span class="tossface">✅</span> You want to keep the UI responsive
- <span class="tossface">✅</span> Operation can fail and needs error handling

**Examples of good async methods:**

- Large array sorting/filtering
- Cryptographic operations (hashing, encryption)
- Complex algorithms (graph traversal, pattern matching)
- Heavy data processing

## Error Handling

### Sync Methods

Sync methods typically use the `throw` macro (alias for `panic!`) for errors:

```rust
fn divide(&mut self, a: Number, b: Number) -> Number {
    if b == 0.0 {
        throw!("Division by zero");
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

Async methods use `promise::reject` utility function for errors:

::: tip
You can also use the `throw!` macro in async methods for error handling
:::

```rust
fn get_user(&mut self, id: Number) -> Promise<User> {
    if id <= 0.0 {
        return promise::reject("Invalid user ID");
    }

    match source.find(id) {
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
| **Error Handling** | `throw!` | Both `throw!` and `promise::reject` |
| **UI Impact** | Blocking | Non-blocking |
| **Use Cases** | Math, formatting | Heavy compute, complex algorithms |
