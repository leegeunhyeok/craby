# Stateful Modules

This guide explains how to create modules that maintain state across method calls.

## Overview

Craby modules can preserve internal Rust state between invocations. Each module instance maintains its own state, allowing you to store and access data across multiple method calls.

Here's a simple storage module that maintains state:

```rust
struct Storage {
    id: usize,
    data: Option<Number>,
}

#[craby_module]
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
