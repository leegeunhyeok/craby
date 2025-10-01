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
impl Spec for Calculator {
    fn add(&self, a: Number, b: Number) -> Number {
        a + b  // Returns immediately
    }

    fn format_string(&self, text: String) -> String {
        text.to_uppercase()  // Returns immediately
    }

    fn is_valid(&self, value: Boolean) -> Boolean {
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

- ✅ Operation completes in **< 16ms** (one frame at 60fps)
- ✅ Simple calculations or data transformations
- ✅ No heavy computations
- ✅ Immediate result is needed

**Examples of good sync methods:**
- Math calculations
- String formatting
- Simple data validation
- Type conversions

## Asynchronous Methods (Promises)

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
impl Spec for HeavyCompute {
    fn calculate_prime(&self, n: Number) -> Promise<Number> {
        if n <= 0.0 {
            return promise::reject("Invalid input");
        }

        // Long-running computation runs in background thread
        let prime = nth_prime(n as i64);
        promise::resolve(prime as f64)
    }

    fn sort_large_array(&self, mut numbers: Array<Number>) -> Promise<Array<Number>> {
        // Heavy sorting operation - runs in background thread
        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
        promise::resolve(numbers)
    }

    fn compute_hash(&self, data: String) -> Promise<String> {
        // CPU-intensive hashing - safe here in background thread
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

- ✅ Operation takes **> 16ms** (would drop frames)
- ✅ CPU-intensive computations
- ✅ You want to keep the UI responsive
- ✅ Operation can fail and needs error handling

**Examples of good async methods:**
- Prime number calculations
- Large array sorting/filtering
- Image/video processing
- Cryptographic operations (hashing, encryption)
- Complex algorithms (graph traversal, pattern matching)
- Heavy data transformations

## Performance Comparison

### Sync Method Performance

```typescript
// ❌ Bad - blocks UI thread for 2 seconds
export interface Spec extends NativeModule {
  slowCalculation(): number;
}
```

```rust
impl Spec for BadExample {
    fn slow_calculation(&self) -> Number {
        // This will FREEZE the UI for 2 seconds!
        thread::sleep(Duration::from_secs(2));
        42.0
    }
}
```

```typescript
console.log('Starting...');
const result = BadExample.slowCalculation();  // UI frozen for 2s
console.log('Done:', result);
```

### Async Method Performance

```typescript
// ✅ Good - runs in background
export interface Spec extends NativeModule {
  slowCalculation(): Promise<number>;
}
```

```rust
impl Spec for GoodExample {
    fn slow_calculation(&self) -> Promise<Number> {
        thread::spawn(|| {
            // This runs in a separate thread
            thread::sleep(Duration::from_secs(2));
            42.0
        });

        promise::resolve(42.0)
    }
}
```

```typescript
console.log('Starting...');
const result = await GoodExample.slowCalculation();  // UI stays responsive!
console.log('Done:', result);
```

## Background Thread Execution

When implementing Promise methods, your Rust code runs in a **background thread** (spawned by C++ layer):

```rust
fn process_data(&self, value: Number) -> Promise<Number> {
    // This runs in a background thread automatically
    // Safe to do heavy computations here
    let result = expensive_calculation(value);
    promise::resolve(result)
}

fn compute_fibonacci(&self, n: Number) -> Promise<Number> {
    // CPU-intensive recursive calculation
    let result = fibonacci(n as u64);
    promise::resolve(result as f64)
}
```

## Error Handling

### Sync Methods

Sync methods typically use **panics** for errors (which crash the app):

```rust
// ❌ Don't do this - will crash the app
fn divide(&self, a: Number, b: Number) -> Number {
    if b == 0.0 {
        panic!("Division by zero");  // Crashes!
    }
    a / b
}
```

**Better approach:** Return nullable or use Promise:

```rust
// ✅ Better - return nullable
fn divide(&self, a: Number, b: Number) -> Nullable<Number> {
    if b == 0.0 {
        return Nullable::none();
    }
    Nullable::some(a / b)
}

// ✅ Best - use Promise for error handling
fn divide(&self, a: Number, b: Number) -> Promise<Number> {
    if b == 0.0 {
        return promise::reject("Cannot divide by zero");
    }
    promise::resolve(a / b)
}
```

### Async Methods

Async methods use **Promise rejections** for errors:

```rust
fn fetch_user(&self, id: Number) -> Promise<User> {
    if id <= 0.0 {
        return promise::reject("Invalid user ID");
    }

    // This runs in background thread (managed by C++ layer)
    match database.find(id) {
        Some(user) => promise::resolve(user),
        None => promise::reject("User not found"),
    }
}
```

```typescript
try {
  const user = await UserService.fetchUser(123);
  console.log(user);
} catch (error) {
  console.error('Failed:', error);
}
```

## Design Patterns

### Pattern 1: Simple Calculation vs Heavy Computation

```typescript
export interface Spec extends NativeModule {
  square(n: number): number;  // Sync - simple operation
  factorial(n: number): Promise<number>;  // Async - heavy computation
}
```

```rust
impl Spec for Calculator {
    fn square(&self, n: Number) -> Number {
        n * n  // Fast calculation
    }

    fn factorial(&self, n: Number) -> Promise<Number> {
        // Runs in background thread - heavy computation
        let result = calculate_factorial(n as u64);
        promise::resolve(result as f64)
    }
}
```

### Pattern 2: Sync Validation, Async Processing

```typescript
export interface Spec extends NativeModule {
  validateInput(data: string): boolean;  // Sync - quick validation
  processInput(data: string): Promise<string>;  // Async - heavy processing
}
```

```rust
impl Spec for Processor {
    fn validate_input(&self, data: String) -> Boolean {
        !data.is_empty() && data.len() < 1000
    }

    fn process_input(&self, data: String) -> Promise<String> {
        // Heavy processing runs in background thread
        let result = complex_transformation(data);
        promise::resolve(result)
    }
}
```

```typescript
const data = "some input data";

// Validate first (sync)
if (!Processor.validateInput(data)) {
  console.error('Invalid input');
  return;
}

// Then process (async)
const result = await Processor.processInput(data);
console.log(result);
```

### Pattern 3: Batch Processing

```typescript
export interface Spec extends NativeModule {
  filterPrimes(numbers: number[]): Promise<number[]>;
}
```

```rust
impl Spec for BatchProcessor {
    fn filter_primes(&self, numbers: Array<Number>) -> Promise<Array<Number>> {
        // Runs in background thread - CPU-intensive filtering
        let primes: Vec<f64> = numbers
            .iter()
            .filter(|&&n| is_prime(n as i64))
            .copied()
            .collect();

        promise::resolve(primes)
    }
}
```

```typescript
// Collect numbers in JavaScript
const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Filter primes in background
const primes = await Processor.filterPrimes(numbers);
console.log('Primes:', primes); // [2, 3, 5, 7]
```

## Combining Sync and Async

### Use Signals for Progress Updates

```typescript
export interface Spec extends NativeModule {
  onProgress: Signal;
  onComplete: Signal;

  processLargeDataset(data: number[]): Promise<number>;
}
```

```rust
impl Spec for DataProcessor {
    fn process_large_dataset(&self, data: Array<Number>) -> Promise<Number> {
        // Runs in background thread
        let chunk_size = data.len() / 100;

        for i in 0..100 {
            // Process chunk
            let start = i * chunk_size;
            let end = ((i + 1) * chunk_size).min(data.len());
            process_chunk(&data[start..end]);

            // Emit progress signal
            self.emit(DataProcessorSignal::OnProgress);
        }

        // Emit completion signal
        self.emit(DataProcessorSignal::OnComplete);

        let result = data.iter().sum();
        promise::resolve(result)
    }
}
```

```typescript
let progress = 0;

DataProcessor.onProgress.addListener(() => {
  progress++;
  console.log(`Progress: ${progress}%`);
});

DataProcessor.onComplete.addListener(() => {
  console.log('Complete!');
});

const largeData = Array.from({ length: 100000 }, (_, i) => i);
const result = await DataProcessor.processLargeDataset(largeData);
console.log('Sum:', result);
```

## Best Practices

### 1. Choose the Right Method Type

```rust
// ✅ Sync - simple math
fn add(&self, a: Number, b: Number) -> Number;

// ✅ Async - heavy computation
fn calculatePrime(&self, n: Number) -> Promise<Number>;

// ❌ Wrong - should be async
fn sortHugeArray(&self, data: Array<Number>) -> Array<Number>;  // Will block!

// ❌ Wrong - should be sync
fn add(&self, a: Number, b: Number) -> Promise<Number>;  // Unnecessary
```

### 2. Keep Sync Methods Fast

```rust
// ✅ Good - completes in microseconds
fn add(&self, a: Number, b: Number) -> Number {
    a + b
}

// ❌ Bad - takes seconds
fn calculate(&self, n: Number) -> Number {
    thread::sleep(Duration::from_secs(2));  // Don't block!
    n
}
```

### 3. Use Async for Heavy Computations

```rust
// ✅ Good - async heavy work
fn sortLargeArray(&self, data: Array<Number>) -> Promise<Array<Number>>;

// ❌ Bad - sync heavy work blocks UI
fn sortLargeArray(&self, data: Array<Number>) -> Array<Number>;
```

### 4. Don't Mix Heavy Work in Sync Methods

```rust
// ❌ Very bad - blocks UI thread
fn compute(&self, n: Number) -> Number {
    expensive_algorithm(n)  // BLOCKS!
}

// ✅ Good - async for heavy work
fn compute(&self, n: Number) -> Promise<Number> {
    // Runs in background thread automatically
    let result = expensive_algorithm(n);
    promise::resolve(result)
}
```

### 5. Document Expected Performance

```typescript
export interface Spec extends NativeModule {
  /**
   * Fast in-memory calculation. Completes in < 1ms.
   */
  add(a: number, b: number): number;

  /**
   * Calculates nth prime number. May take several seconds for large n.
   * Executes in background thread to keep UI responsive.
   */
  calculatePrime(n: number): Promise<number>;
}
```

## Summary

| Aspect | Sync | Async (Promise) |
|--------|------|-----------------|
| **Execution** | JS thread | Background thread |
| **Returns** | Direct value | Promise |
| **Duration** | < 16ms | Any duration |
| **Heavy Work** | Avoid | Perfect for |
| **Error Handling** | Panic | Promise rejection |
| **UI Impact** | Can block | Non-blocking |
| **Use Cases** | Math, formatting | Heavy compute, complex algorithms |

## Next Steps

- [Types](/guide/types) - Learn about type system
- [Errors](/guide/errors) - Error handling strategies
- [Signals](/guide/signals) - Event notifications
