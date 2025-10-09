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
impl MyModuleSpec for MyModule {
    fn start_process(&mut self) -> Void {
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

## Subscribing to Signals in JavaScript

Subscribe to signals to invoke with callback:

```typescript
import { MyModule } from 'your-module';

// Add a listener
const cleanup = MyModule.onDataReceived(() => {
  console.log('Data received from native!');
});

// Remove the listener when done
cleanup();
```

### Multiple Listeners

You can add multiple listeners to the same signal:

```typescript
MyModule.onProgress(() => {
  console.log('Progress update 1');
});

MyModule.onProgress(() => {
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
    const cleanup = MyModule.onDataReceived(() => {
      console.log('Data received!');
    });

    // Cleanup listener on unmount
    return () => cleanup();
  }, []);

  return <View>...</View>;
}
```

## Limitations

<div class="tossface">

- ❌ Signals cannot carry data

</div>
