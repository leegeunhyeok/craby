# Signals

Signals enable one-way communication from native Rust code to JavaScript, allowing you to emit events that trigger callbacks in your React Native app.

## What are Signals?

Signals are simple event notifications sent from Rust to JavaScript. Unlike method calls that go from JS to native, signals flow in the opposite direction—from native to JS.

- **One-way**: Rust → JavaScript only
- **No data payload**: Signals don't carry data (they just trigger callbacks)
- **Multiple listeners**: JavaScript can register multiple listeners for the same signal
- **Asynchronous**: Signals are emitted asynchronously and don't block native code

## Defining Signals

Define signals as properties with the `Signal` type in your TypeScript spec:

```typescript
import type { NativeModule, Signal } from 'craby-modules';

export interface Spec extends NativeModule {
  // Signal definitions
  onStarted: Signal;
  onFinished: Signal;

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
#[craby_module]
impl MyModuleSpec for MyModule {
    fn start_process(&mut self) -> Void {
        // Emit a signal to notify JavaScript
        self.emit(MyModuleSignal::OnStarted);

        // Do some work...
        process_data();

        // Emit completion signal
        self.emit(MyModuleSignal::OnFinished);
    }
}
```

### Generated Signal Enum

Craby automatically generates a Signal enum for your module:

```rust
// Auto-generated
pub enum ProcessModuleSignal {
    OnStarted,
    OnFinished,
}
```

## Subscribing to Signals in JavaScript

Subscribe to signals to invoke with callback:

```typescript
import { ProcessModule } from 'your-module';

// Add a listener
const cleanup = ProcessModule.onStarted(() => {
  console.log('Callback invoked from native!');
});

// Remove the listener when done
cleanup();
```

### Multiple Listeners

You can add multiple listeners to the same signal:

```typescript
MyModule.onFinished(() => {
  console.log('Finished 1');
});

MyModule.onFinished(() => {
  console.log('Finished 2');
});

// Both listeners will be called when the signal is emitted
```

## Limitations

Signals are designed to invoke JavaScript callback functions from Rust. As such, they cannot carry data payloads—they serve only as event notifications to trigger JavaScript callbacks.
