# Errors

This guide covers error handling in Craby modules using Promise rejections and exceptions.

## Error Handling Strategies

Craby provides two ways to handle errors:

1. **Panics** - For synchronous errors that propagate to JavaScript
2. **Promise Rejections** - For recoverable errors in async operations

## Panics

Use the `throw!` macro (alias for `panic!`) to throw exceptions that propagate to JavaScript. The panic is handled by `panic::catch_unwind()` and the error is sent to C++ safely.

In the C++ layer, a runtime exception is created and thrown to the JavaScript runtime via `jsi::JSError`.

### Exception Flow

```
Panic occurred → `std::panic::catch_unwind()` → C++ Exception → `jsi::JSError`
```

### Throwing Exceptions for Synchronous Errors

```rust
#[craby_module]
impl CalculatorSpec for Calculator {
    fn divide(&mut self, a: Number, b: Number) -> Number {
        if b == 0.0 {
            throw!("Division by zero"); // Throws to JavaScript!
        }
        a / b
    }
}
```

> Exceptions thrown via `throw!` will appear as JavaScript errors and can be caught with try-catch blocks in JavaScript:

```typescript
try {
  CalculatorModule.divide(10, 0);
} catch (error) {
  console.error('Error:', error.message);
}
```

## Promise Rejections

Use Promise rejections for recoverable errors that JavaScript can handle.

### Basic Usage

```typescript
export interface Spec extends NativeModule {
  parseLargeData(data: string): Promise<void>;
}
```

```rust
#[craby_module]
impl DataParserSpec for DataParser {
    fn parse_large_data(&mut self, data: &str) -> Promise<Void> {
        if data.is_empty() {
            return promise::reject("Data cannot be empty");
        }

        match parse(data) {
            Ok(_) => promise::resolve(()),
            Err(e) => promise::reject(&e.to_string()),
        }
    }
}
```

### JavaScript Error Handling

```typescript
try {
  const result = await DataParser.parseLargeData(data);
  console.log(result);
} catch (error) {
  console.error('Failed to parse data:', error);
}

// Or with .catch()
DataParser.parseLargeData(data)
  .then(data => console.log(data))
  .catch(error => console.error('Error:', error));
```

## Summary

| Strategy | Use Case |
|----------|----------|
| **Panic** | Sync immediate errors |
| **Promise Rejection** | Recoverable errors |
