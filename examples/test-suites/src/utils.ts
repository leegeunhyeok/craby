export function toErrorObject(error: any) {
  return { message: error.message };
}

export function stringToBytes(str: string) {
  const bytes = new Uint8Array(str.length);
  for (let i = 0; i < str.length; i++) {
    bytes[i] = str.charCodeAt(i);
  }
  return bytes;
}

export function createTaskHandler<T>() {
  let resolver: (value: T) => void;
  let rejector: (reason: unknown) => void;

  const task = new Promise<T>((resolve, reject) => {
    resolver = resolve;
    rejector = reject;
  });

  return Object.defineProperties(task, {
    resolver: {
      value: (value: T) => resolver?.(value),
    },
    rejector: {
      value: (reason: unknown) => rejector?.(reason),
    },
  }) as Promise<T> & { resolver: (value: T) => void; rejector: (reason: unknown) => void };
}

/**
 * Callback test helper function.
 *
 * Signal callbacks in Craby are invoked via `callInvoker.invokeAsync()`, which schedules execution in the microtask queue.
 * To properly test these callbacks, we must ensure test assertions run after the microtask queue has been processed.
 *
 * **Why `setTimeout`?**
 * - Microtasks execute before the next macrotask
 * - `setTimeout(fn, 0)` schedules execution in the macrotask queue
 *
 * @param fn - The function to execute after the next tick.
 * @returns
 */
export function nextTick(fn: () => void) {
  return setTimeout(fn, 0);
}

/**
 * Waits for all expected signals to be emitted and processed.
 * Polls until the expected count is reached or timeout occurs.
 */
export function waitForSignals(
  checkFn: () => { current: number; expected: number },
  timeout: number = 5000,
  interval: number = 50,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const startTime = Date.now();

    const check = () => {
      const { current, expected } = checkFn();

      if (current >= expected) {
        resolve();
        return;
      }

      if (Date.now() - startTime > timeout) {
        reject(new Error(`Timeout: Expected ${expected} signals, got ${current} after ${timeout}ms`));
        return;
      }

      setTimeout(check, interval);
    };

    check();
  });
}
