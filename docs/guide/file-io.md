# File I/O

This guide explains how to perform file I/O operations in Craby modules using the application's data path.

## Overview

Craby modules can access and store files within the application's data directory. The `ctx.data_path` field in your module's context provides the path to this directory, where you can safely read and write files.

## Accessing the Data Path

Every Craby module receives a context (`ctx`) that includes `data_path`:

```rust
#[craby_module]
impl MyModule {
    fn get_file_path(&self) -> String {
        // `self.ctx.data_path` is application's data path
        self.ctx.data_path.clone()
    }
}
```

The `data_path` points to the application's designated data storage directory, which varies by platform:

- **Android**: Typically `/data/data/<package-name>/files/`
- **iOS**: The app's Documents directory

## Reading and Writing Files

Here's a complete example of a module that reads and writes data:

```rust
#[craby_module]
impl StorageSpec for Storage {
    fn get_data_path(&mut self) -> String {
        self.ctx.data_path.clone()
    }

    fn write_data(&mut self, value: &str) -> Boolean {
        std::fs::write(self.get_file_path(), value).is_ok()
    }

    fn read_data(&mut self) -> Nullable<String> {
        match std::fs::read_to_string(self.get_file_path()) {
            Ok(data) => Nullable::<String>::some(data),
            Err(_) => Nullable::<String>::none(),
        }
    }
}
```

### TypeScript Usage

```typescript
// Get the data path
const dataPath = Storage.getDataPath();
console.log('Data path:', dataPath);

// Write data to file
const success = Storage.writeData('Hello, world!');
console.log('Write successful:', success);

// Read data from file
const data = Storage.readData();
if (data !== null) {
  console.log('Read data:', data);
} else {
  console.log('No data found');
}
```

## Limitations

::: warning
- Only access files within the `data_path` directory
- Do not attempt to access files outside this directory, as it may fail due to platform-specific permissions
- File operations are blocking and should be used carefully to avoid blocking the JavaScript thread
:::

For long-running file operations, consider using [async methods](/guide/sync-vs-async) to avoid blocking the UI.
