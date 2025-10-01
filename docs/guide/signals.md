# Signals

Signals enable one-way communication from native Rust code to JavaScript, allowing you to emit events that trigger callbacks in your React Native app.

## What are Signals?

Signals are simple event notifications sent from Rust to JavaScript. Unlike method calls that go from JS to native, signals flow in the opposite direction—from native to JS.

**Key characteristics:**
- **One-way**: Native → JavaScript only
- **No data payload**: Signals don't carry data (just trigger callbacks)
- **Multiple listeners**: JavaScript can register multiple listeners for the same signal
- **Asynchronous**: Signals are emitted asynchronously and don't block native code

## Defining Signals

Define signals as properties with the `Signal` type in your TypeScript spec:

```typescript
import type { NativeModule, Signal } from 'craby-modules';

export interface Spec extends NativeModule {
  // Signal definitions
  onDataReceived: Signal;
  onProgress: Signal;
  onError: Signal;
  onComplete: Signal;

  // Regular methods
  startProcess(): void;
  stopProcess(): void;
}
```

::: info Signal Names
The property name (e.g., `onDataReceived`) becomes the signal name. Use descriptive names that clearly indicate when the signal is emitted.
:::

## Emitting Signals from Rust

In your Rust implementation, emit signals using the `emit()` method:

```rust
impl Spec for MyModule {
    fn start_process(&self) -> Void {
        // Emit a signal to notify JavaScript
        self.emit(MyModuleSignal::OnProgress);

        // Do some work...
        process_data();

        // Emit completion signal
        self.emit(MyModuleSignal::OnComplete);
    }
}
```

### Generated Signal Enum

Craby automatically generates a signal enum for your module:

```rust
// Auto-generated
pub enum MyModuleSignal {
    OnDataReceived,
    OnProgress,
    OnError,
    OnComplete,
}
```

## Listening to Signals in JavaScript

Subscribe to signals using the `addListener()` method:

```typescript
import { MyModule } from 'your-module';

// Add a listener
const subscription = MyModule.onDataReceived.addListener(() => {
  console.log('Data received from native!');
  // Update UI, fetch data, etc.
});

// Remove the listener when done
subscription.remove();
```

### Multiple Listeners

You can add multiple listeners to the same signal:

```typescript
MyModule.onProgress.addListener(() => {
  console.log('Progress update 1');
});

MyModule.onProgress.addListener(() => {
  console.log('Progress update 2');
});

// Both listeners will be called when the signal is emitted
```

### Using with React Hooks

```tsx
import { useEffect } from 'react';
import { MyModule } from 'your-module';

function MyComponent() {
  useEffect(() => {
    const subscription = MyModule.onDataReceived.addListener(() => {
      console.log('Data received!');
    });

    // Cleanup listener on unmount
    return () => subscription.remove();
  }, []);

  return <View>...</View>;
}
```

## Use Cases

### 1. Background Task Completion

```typescript
// TypeScript
export interface Spec extends NativeModule {
  onTaskComplete: Signal;
  onTaskError: Signal;

  startBackgroundTask(duration: number): Promise<void>;
}
```

```rust
// Rust
impl Spec for BackgroundWorker {
    fn start_background_task(&self, duration: Number) -> Promise<Void> {
        // Runs in background thread (managed by C++ layer)
        std::thread::sleep(std::time::Duration::from_secs(duration as u64));

        // Emit completion signal
        self.emit(BackgroundWorkerSignal::OnTaskComplete);

        promise::resolve(())
    }
}
```

```typescript
// JavaScript
MyModule.onTaskComplete.addListener(() => {
  console.log('Background task finished!');
});

MyModule.onTaskError.addListener(() => {
  console.error('Background task failed!');
});

await MyModule.startBackgroundTask(5);
```

### 2. Progress Notifications

```typescript
export interface Spec extends NativeModule {
  onProgress: Signal;
  onComplete: Signal;

  processLargeFile(path: string): void;
}
```

```rust
impl Spec for FileProcessor {
    fn process_large_file(&self, path: String) -> Void {
        let total_chunks = 100;

        for i in 0..total_chunks {
            // Process chunk...
            process_chunk(i);

            // Emit progress signal
            self.emit(FileProcessorSignal::OnProgress);
        }

        // Emit completion
        self.emit(FileProcessorSignal::OnComplete);
    }
}
```

```typescript
let progressCount = 0;

MyModule.onProgress.addListener(() => {
  progressCount++;
  console.log(`Progress: ${progressCount}%`);
});

MyModule.onComplete.addListener(() => {
  console.log('File processing complete!');
  progressCount = 0;
});
```

### 3. Real-Time Event Streams

For continuous event streams:

```typescript
export interface Spec extends NativeModule {
  onSensorData: Signal;

  startSensorMonitoring(durationMs: number): Promise<void>;
}
```

```rust
impl Spec for SensorMonitor {
    fn start_sensor_monitoring(&self, duration_ms: Number) -> Promise<Void> {
        // Runs in background thread
        let iterations = (duration_ms / 100.0) as i32;

        for _ in 0..iterations {
            // Read sensor data
            let data = read_sensor();

            // Emit signal on each reading
            self.emit(SensorMonitorSignal::OnSensorData);

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        promise::resolve(())
    }
}
```

### 4. Error Notifications

```typescript
export interface Spec extends NativeModule {
  onNetworkError: Signal;
  onAuthError: Signal;

  fetchData(url: string): Promise<string>;
}
```

```rust
impl Spec for NetworkClient {
    fn fetch_data(&self, url: String) -> Promise<String> {
        match fetch(&url) {
            Ok(data) => promise::resolve(data),
            Err(e) => {
                // Emit error signal
                match e.kind() {
                    ErrorKind::Network => {
                        self.emit(NetworkClientSignal::OnNetworkError);
                    }
                    ErrorKind::Auth => {
                        self.emit(NetworkClientSignal::OnAuthError);
                    }
                }

                promise::reject("Failed to fetch data")
            }
        }
    }
}
```

## Signals vs Promises

Choose between signals and promises based on your use case:

### Use Signals When:

- ✅ You need to notify JS multiple times
- ✅ You don't need to pass data with the notification
- ✅ Events can occur independently of method calls
- ✅ You need fire-and-forget notifications

### Use Promises When:

- ✅ You need to return data to JavaScript
- ✅ Operation has a single completion (success or failure)
- ✅ Caller needs to know when operation finishes
- ✅ You need error handling with data

**Example comparison:**

```typescript
// ✅ Good use of Signal
export interface Spec extends NativeModule {
  onHeartbeat: Signal;  // Fires periodically, no data needed
  startMonitoring(): void;
}

// ✅ Good use of Promise
export interface Spec extends NativeModule {
  fetchUser(id: number): Promise<User>;  // Returns data once
}

// ❌ Wrong - should use Promise
export interface Spec extends NativeModule {
  onUserFetched: Signal;  // Trying to pass data via signal
  fetchUser(id: number): void;
}

// ❌ Wrong - should use Signal
export interface Spec extends NativeModule {
  getHeartbeat(): Promise<void>;  // Polling for events
}
```

## Limitations

### No Data Payload

Signals cannot carry data. If you need to pass data, use one of these approaches:

**Option 1: Use Promises**
```typescript
export interface Spec extends NativeModule {
  fetchData(id: number): Promise<UserData>;
}
```

**Option 2: Call a method after signal**
```typescript
export interface Spec extends NativeModule {
  onDataReady: Signal;
  getData(): UserData;
}
```

```typescript
MyModule.onDataReady.addListener(() => {
  const data = MyModule.getData();  // Fetch data after signal
  console.log(data);
});
```

**Option 3: Use state management**
```typescript
export interface Spec extends NativeModule {
  onStateChanged: Signal;
  getState(): AppState;
}
```

### Emitting Signals from Async Methods

When you need to emit signals during async operations, use Promise methods:

```rust
// ✅ Correct - emit from async method
fn start_task(&self) -> Promise<Void> {
    // Runs in background thread automatically
    // Process work...
    do_work();

    // Emit signal when done
    self.emit(MySignal::OnComplete);

    promise::resolve(())
}

// ✅ Also correct - emit from sync method (if fast)
fn trigger_event(&self) -> Void {
    // Fast operation on main thread
    self.emit(MySignal::OnEvent);
}
```

## Best Practices

### 1. Use Descriptive Names

```typescript
// ✅ Good
onConnectionEstablished: Signal;
onDataSyncComplete: Signal;
onAuthenticationFailed: Signal;

// ❌ Less clear
onEvent1: Signal;
onUpdate: Signal;
onSignal: Signal;
```

### 2. Group Related Signals

```typescript
// ✅ Good organization
export interface Spec extends NativeModule {
  // Download lifecycle
  onDownloadStarted: Signal;
  onDownloadProgress: Signal;
  onDownloadComplete: Signal;
  onDownloadFailed: Signal;

  // Methods
  startDownload(url: string): void;
}
```

### 3. Clean Up Listeners

```typescript
// ✅ Good - cleanup on unmount
useEffect(() => {
  const sub = MyModule.onData.addListener(() => {});
  return () => sub.remove();
}, []);

// ❌ Memory leak - no cleanup
useEffect(() => {
  MyModule.onData.addListener(() => {});
}, []);
```

### 4. Document Signal Behavior

```typescript
export interface Spec extends NativeModule {
  /**
   * Emitted when new sensor data is available.
   * Fires approximately every 100ms while monitoring is active.
   */
  onSensorUpdate: Signal;

  /**
   * Emitted once when monitoring stops, either by calling
   * stopMonitoring() or due to an error.
   */
  onMonitoringStopped: Signal;

  startMonitoring(): void;
  stopMonitoring(): void;
}
```

## Next Steps

- [Types](/guide/types) - Learn about type system
- [Errors](/guide/errors) - Error handling patterns
- [Sync vs Async](/guide/sync-vs-async) - Async operations with Promises
