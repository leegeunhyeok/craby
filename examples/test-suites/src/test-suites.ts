import * as Module from 'craby-test';
import { assert, isEqual } from 'es-toolkit';
import type { TestSuite } from './types';
import { createTaskHandler, nextTick, stringToBytes, toErrorObject, waitForSignals } from './utils';

function assertBufferBytes(actual: ArrayBuffer, expected: Uint8Array) {
  assert(actual instanceof ArrayBuffer, 'Result is not an ArrayBuffer');
  const bytes = new Uint8Array(actual);
  assert(bytes.length === expected.length, 'ArrayBuffer length is incorrect');
  assert(
    bytes.every((byte, index) => byte === expected[index]),
    'ArrayBuffer contents are incorrect',
  );
}

function assertBufferObject(actual: Module.BufferObject, expected: Module.BufferObject) {
  assert(actual.label === expected.label, 'Buffer object label is incorrect');
  assertBufferBytes(actual.bytes, new Uint8Array(expected.bytes));
  assert(isEqual(actual.values, expected.values), 'Buffer object values are incorrect');
  if (expected.nested === null) {
    assert(actual.nested === null, 'Nested buffer object should be null');
  } else {
    assert(actual.nested !== null, 'Nested buffer object is missing');
    if (expected.nested.bytes === null) {
      assert(actual.nested.bytes === null, 'Nested buffer should be null');
    } else {
      assert(actual.nested.bytes !== null, 'Nested buffer is missing');
      assertBufferBytes(actual.nested.bytes, new Uint8Array(expected.nested.bytes));
    }
  }
}

const TEST_SUITES: TestSuite[] = [
  {
    label: 'Number',
    action: () => Module.CrabyTestModule.numericMethod(123),
  },
  {
    label: 'Boolean',
    action: () => Module.CrabyTestModule.booleanMethod(true),
  },
  {
    label: 'String',
    action: () => Module.CrabyTestModule.stringMethod('Hello, World!'),
  },
  {
    label: 'Object',
    action: () =>
      Module.CrabyTestModule.objectMethod({
        foo: 'foo',
        bar: 123,
        baz: false,
        sub: {
          a: 'a',
          b: 456,
          c: true,
        },
        points: [
          { x: 1, y: 2 },
          { x: 3, y: 4 },
        ],
        camelCase: 0,
        PascalCase: 0,
        snake_case: 0,
      }),
  },
  {
    label: 'Object',
    description: '(Invalid object)',
    action: () => {
      try {
        return Module.CrabyTestModule.objectMethod({ foo: 123 } as any);
      } catch (error: unknown) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Object',
    description: '(Nullable 1)',
    action: () => {
      try {
        return Module.CrabyTestModule.objectMethod({
          foo: 'foo',
          bar: 456,
          baz: true,
          sub: null,
          points: [{ x: 1, y: 2 }],
          camelCase: 0,
          PascalCase: 0,
          snake_case: 0,
        });
      } catch (error: unknown) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Object',
    description: '(Nullable 2)',
    action: () => {
      try {
        return Module.CrabyTestModule.objectMethod({
          foo: 'foo',
          bar: 456,
          baz: true,
          sub: {
            a: null,
            b: 789,
            c: false,
          },
          points: [{ x: 1, y: 2 }],
          camelCase: 0,
          PascalCase: 0,
          snake_case: 0,
        });
      } catch (error: unknown) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'ArrayBuffer',
    action: () => {
      const text = 'Hello, World!';
      const arrayBuffer = stringToBytes(text);
      const result = Module.CrabyTestModule.arrayBufferMethod(arrayBuffer.buffer);
      const type = result instanceof ArrayBuffer ? 'ArrayBuffer' : '';

      assert(type === 'ArrayBuffer', '`arrayBufferMethod` result is not an ArrayBuffer');
      assertBufferBytes(
        result,
        arrayBuffer.map((byte) => byte ^ 255),
      );
      assertBufferBytes(arrayBuffer.buffer, stringToBytes(text));

      return {
        text,
        type,
        data: Array.from(new Uint8Array(result)),
      };
    },
  },
  {
    label: 'ArrayBuffer',
    description: '(Ownership, empty and large buffers)',
    action: () => {
      const sizes = [0, 17, 256 * 1024];
      for (const size of sizes) {
        const input = Uint8Array.from({ length: size }, (_, index) => index % 251);
        const original = input.slice();
        const results: Uint8Array[] = [];
        for (let repeat = 0; repeat < 3; repeat++) {
          const result = Module.CrabyTestModule.arrayBufferMethod(input.buffer);
          assert(result !== input.buffer, 'Result aliases the input ArrayBuffer');
          assertBufferBytes(
            result,
            original.map((byte) => byte ^ 255),
          );
          assertBufferBytes(input.buffer, original);
          const bytes = new Uint8Array(result);
          if (size > 0) {
            bytes[0] = repeat;
            assert(bytes[0] === repeat, 'Result ArrayBuffer is not writable');
            assertBufferBytes(input.buffer, original);
          }
          results.push(bytes);
        }
        for (const [repeat, bytes] of results.entries()) {
          if (size > 0) {
            assert(bytes[0] === repeat, 'A later call changed a previous ArrayBuffer result');
          }
        }
      }
      return { sizes, repeats: 3 };
    },
  },
  {
    label: 'Object',
    description: '(Nested and nullable ArrayBuffers)',
    action: () => {
      const inputs: Module.BufferObject[] = [
        {
          label: 'nested',
          bytes: new Uint8Array([0, 127, 255]).buffer,
          values: [2, 4],
          nested: { bytes: new Uint8Array([1, 128, 254]).buffer },
        },
        { label: 'null buffer', bytes: new ArrayBuffer(0), values: [], nested: { bytes: null } },
        { label: 'null object', bytes: new Uint8Array([3]).buffer, values: [0], nested: null },
      ];
      for (const input of inputs) {
        const original: Module.BufferObject = {
          ...input,
          bytes: input.bytes.slice(0),
          values: [...input.values],
          nested: input.nested === null ? null : { bytes: input.nested.bytes?.slice(0) ?? null },
        };
        const result = Module.CrabyTestModule.bufferObjectMethod(input);
        assertBufferObject(result, {
          label: `From Rust: ${original.label}`,
          bytes: new Uint8Array(original.bytes).map((byte) => byte ^ 255).buffer,
          values: [...original.values, 1, 2, 3].map((value) => value * 2),
          nested:
            original.nested === null
              ? null
              : {
                  bytes:
                    original.nested.bytes === null
                      ? null
                      : new Uint8Array(original.nested.bytes).map((byte) => byte ^ 255).buffer,
                },
        });
        new Uint8Array(result.bytes).fill(42);
        if (result.nested?.bytes) {
          new Uint8Array(result.nested.bytes).fill(42);
        }
        result.values[0] = -1;
        assertBufferObject(input, original);
      }
      return { cases: inputs.length };
    },
  },
  {
    label: 'Array',
    description: '(Custom struct)',
    action: () => {
      const result = Module.CrabyTestModule.customStructArrayMethod([
        { x: 1, y: 2 },
        { x: 3, y: 4 },
      ]);

      assert(
        isEqual(result, [
          { x: 2, y: 3 },
          { x: 4, y: 5 },
          { x: 9, y: 9 },
        ]),
        '`customStructArrayMethod` result is incorrect',
      );

      return result;
    },
  },
  {
    label: 'Array',
    action: () => Module.CrabyTestModule.arrayMethod([1, 2, 3]),
  },
  {
    label: 'Enum',
    action: () => Module.CrabyTestModule.enumMethod(Module.MyEnum.Foo, Module.SwitchState.Off),
  },
  {
    label: 'Enum',
    description: '(Invalid string enum value)',
    action: () => {
      try {
        return Module.CrabyTestModule.enumMethod('UNKNOWN' as any, Module.SwitchState.Off);
      } catch (error: any) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Enum',
    description: '(Invalid numeric enum value)',
    action: () => {
      try {
        return Module.CrabyTestModule.enumMethod(Module.MyEnum.Baz, -999 as any);
      } catch (error: any) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Nullable',
    description: '(Non null)',
    action: () => Module.CrabyTestModule.nullableMethod(123),
  },
  {
    label: 'Nullable',
    description: '(Null -> Non null)',
    action: () => Module.CrabyTestModule.nullableMethod(null),
  },
  {
    label: 'Nullable',
    description: '(Non null -> Null)',
    action: () => Module.CrabyTestModule.nullableMethod(-123),
  },
  {
    label: 'Promise',
    action: () => Module.CrabyTestModule.promiseMethod(123),
  },
  {
    label: 'Promise',
    description: '(Rejected promise)',
    action: () => Module.CrabyTestModule.promiseMethod(-123).catch((error) => toErrorObject(error)),
  },
  {
    label: 'Promise',
    description: '(String arg)',
    action: async () => {
      const paths = ['', 'Hello, Craby!', 'craby/'.repeat(16 * 1024)];
      for (const path of paths) {
        const len = await Module.CrabyTestModule.promiseStringMethod(path);
        assert(len === path.length, `promiseStringMethod returned wrong length: ${len}`);
      }
      return { lengths: paths.map((path) => path.length) };
    },
  },
  {
    label: 'Promise',
    description: '(Owned ArrayBuffer arguments and results)',
    action: async () => {
      const sizes = [0, 17, 256 * 1024];
      for (const size of sizes) {
        for (let repeat = 0; repeat < 2; repeat++) {
          const input = Uint8Array.from({ length: size }, (_, index) => (index + repeat) % 251);
          const expected = input.map((byte) => byte ^ 255);
          const pending = Module.CrabyTestModule.promiseArrayBufferMethod(input.buffer);
          input.fill(42);
          const result = await pending;
          assert(result !== input.buffer, 'Promise result aliases the input ArrayBuffer');
          assertBufferBytes(result, expected);
          const bytes = new Uint8Array(result);
          if (size > 0) {
            bytes[0] = 7;
            assert(bytes[0] === 7, 'Promise result ArrayBuffer is not writable');
          }
          assert(
            input.every((byte) => byte === 42),
            'Promise changed the input ArrayBuffer',
          );
        }
      }
      return { sizes, repeats: 2 };
    },
  },
  {
    label: 'Promise',
    description: '(Owned object arguments and results)',
    action: async () => {
      for (let repeat = 0; repeat < 3; repeat++) {
        const input: Module.BufferObject = {
          label: 'queued',
          bytes: new Uint8Array([0, 127, 255]).buffer,
          values: [2, 4],
          nested: { bytes: new Uint8Array([1, 128, 254]).buffer },
        };
        const pending = Module.CrabyTestModule.promiseBufferObjectMethod(input);
        input.label = 'changed';
        new Uint8Array(input.bytes).fill(42);
        input.values.fill(42);
        assert(input.nested !== null && input.nested.bytes !== null, 'Missing test buffer');
        new Uint8Array(input.nested.bytes).fill(42);
        const result = await pending;
        assertBufferObject(result, {
          label: 'From Rust: queued',
          bytes: new Uint8Array([255, 128, 0]).buffer,
          values: [4, 8, 2, 4, 6],
          nested: { bytes: new Uint8Array([254, 127, 1]).buffer },
        });
        new Uint8Array(result.bytes).fill(7);
        assert(result.nested !== null && result.nested.bytes !== null, 'Missing result buffer');
        new Uint8Array(result.nested.bytes).fill(7);
        result.values[0] = 7;
        assertBufferObject(input, {
          label: 'changed',
          bytes: new Uint8Array([42, 42, 42]).buffer,
          values: [42, 42],
          nested: { bytes: new Uint8Array([42, 42, 42]).buffer },
        });
      }
      return { repeats: 3 };
    },
  },
  {
    label: 'State',
    action: () => {
      const data = Date.now();

      Module.CrabyTestModule.setState(data);

      const state = Module.CrabyTestModule.getState();
      assert(state === data, '`getState` result is incorrect');

      return { data, state };
    },
  },
  {
    label: 'Context',
    description: '(Data path)',
    action: () => {
      const path = Module.CrabyTestModule.getDataPath();

      assert(path !== '', '`getDataPath` result is empty');

      return path;
    },
  },
  {
    label: 'File I/O',
    action: () => {
      const data = 'Hello, World!';

      const writeResult = Module.CrabyTestModule.writeData(data);
      assert(writeResult, '`writeData` result is false');

      const readData = Module.CrabyTestModule.readData();
      assert(readData === data, '`readData` result is incorrect');

      return { write: writeResult, read: readData };
    },
  },
  {
    label: 'Panics',
    action: () => {
      try {
        return Module.CalculatorModule.divide(10, 0);
      } catch (error: unknown) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Panics',
    description: '(in Promise)',
    action: () => Module.CrabyTestModule.promiseMethod(0).catch((error) => toErrorObject(error)),
  },
  {
    label: 'Signal',
    action: async () => {
      let invoked = 0;
      const TRIGGER_COUNT = 3;
      const task = createTaskHandler<object>();

      const cleanup = Module.CrabyTestModule.onSignal(() => {
        ++invoked;
      });

      for (let i = 0; i < TRIGGER_COUNT; i++) {
        await Module.CrabyTestModule.triggerSignal();
      }

      const cleanupResults = [
        cleanup(),
        cleanup(), // noop
        cleanup(), // noop
      ];

      assert(
        cleanupResults.every((result) => result === undefined),
        '`cleanup` results are not undefined',
      );

      // Trigger signal after the cleanup is called
      await Module.CrabyTestModule.triggerSignal();

      nextTick(() => {
        if (invoked === TRIGGER_COUNT) {
          task.resolver({ invoked });
        } else {
          task.rejector(new Error(`Expected callback to be called ${TRIGGER_COUNT} times, got ${invoked}`));
        }
      });

      return task;
    },
  },
  {
    label: 'Signal',
    description: 'Multiple listeners',
    action: async () => {
      let invoked = 0;
      const LISTENER_COUNT = 3;
      const TRIGGER_COUNT = 3;
      const task = createTaskHandler<object>();

      const cleanupFunctions = Array.from({ length: LISTENER_COUNT }, () => {
        return Module.CrabyTestModule.onSignal(() => {
          ++invoked;
        });
      });

      const cleanup = () => {
        cleanupFunctions.forEach((cleanup) => {
          cleanup();
        });
      };

      for (let i = 0; i < TRIGGER_COUNT; i++) {
        await Module.CrabyTestModule.triggerSignal();

        await waitForSignals(() => ({ current: invoked, expected: (i + 1) * LISTENER_COUNT }), 500);
      }

      cleanup();
      cleanup(); // noop
      cleanup(); // noop

      // Trigger signal after the cleanup is called
      await Module.CrabyTestModule.triggerSignal();

      await new Promise((resolve) => setTimeout(resolve, 100));

      await new Promise<void>((resolve) => {
        nextTick(() => {
          const expected = TRIGGER_COUNT * LISTENER_COUNT;
          if (invoked === expected) {
            task.resolver({ invoked });
          } else {
            task.rejector(new Error(`Expected callback to be called ${expected} times, got ${invoked}`));
          }
          resolve();
        });
      });

      return task;
    },
  },
  {
    label: 'Signal',
    description: 'onProgress, onError',
    action: async () => {
      const task = createTaskHandler<object>();

      let invoked = 0;
      let errorInvoked = 0;
      const assertProgressData = [
        { progress: 0 },
        { progress: 1 },
        { progress: 2 },
        { progress: 3 },
        { progress: 4 },
        { progress: 5 },
        { progress: 6 },
        { progress: 7 },
        { progress: 8 },
        { progress: 9 },
      ];
      const assertErrorData = [{ reason: 'Error' }];
      const onProgressData: Module.ProgressEvent[] = [];
      const onErrorData: Module.MyModuleError[] = [];

      // Test that onProgress signal can be registered
      const onProgress = Module.CrabyTestModule.onProgress((data) => {
        onProgressData.push(data);
      });

      const onError = Module.CrabyTestModule.onError((data) => {
        onErrorData.push(data);
        ++errorInvoked;
      });

      const cleanup = Module.CrabyTestModule.onSignal(() => {
        ++invoked;
      });

      await Module.CrabyTestModule.triggerSignal();

      assert(typeof cleanup === 'function', '`onSignal` cleanup is not a function');
      assert(typeof onProgress === 'function', '`onProgress` cleanup is not a function');
      assert(typeof onError === 'function', '`onError` cleanup is not a function');

      // Wait for async callbacks to complete
      nextTick(() => {
        const cleanupResult = cleanup();
        assert(cleanupResult === undefined, '`cleanup` result is not undefined');

        const onProgressResult = onProgress();
        assert(onProgressResult === undefined, '`onProgress` cleanup result is not undefined');

        const onErrorResult = onError();
        assert(onErrorResult === undefined, '`onError` cleanup result is not undefined');

        assert(isEqual(onProgressData, assertProgressData), '`onProgress` data is incorrect');
        assert(isEqual(onErrorData, assertErrorData), '`onError` data is incorrect');

        // Test multiple cleanups (should be noop)
        cleanup();
        onProgress();
        onError();

        if (invoked === 1 && errorInvoked === 1 && onProgressData.length === 10) {
          task.resolver({ invoked, errorInvoked, onProgressCount: onProgressData.length });
        } else {
          task.rejector(
            new Error(
              `Expected onSignal: 1, onProgress: 10, onError: 1, got onSignal: ${invoked}, onProgress: ${onProgressData.length}, onError: ${errorInvoked}`,
            ),
          );
        }
      });

      return task;
    },
  },
  {
    label: 'Multiple TurboModules',
    description: 'Calculator',
    action: () => {
      const a = 5;
      const b = 10;

      const add = Module.CalculatorModule.add(a, b);
      const subtract = Module.CalculatorModule.subtract(a, b);
      const multiply = Module.CalculatorModule.multiply(a, b);
      const divide = Module.CalculatorModule.divide(a, b);

      assert(add === a + b, '`add` result is incorrect');
      assert(subtract === a - b, '`subtract` result is incorrect');
      assert(multiply === a * b, '`multiply` result is incorrect');
      assert(divide === a / b, '`divide` result is incorrect');

      return { add, subtract, multiply, divide };
    },
  },
  {
    label: 'Conventions',
    action: () => {
      type MethodResult = { invoked: boolean; typeof: null | string };

      let camelMethod: MethodResult = { invoked: false, typeof: null };
      let pascalMethod: MethodResult = { invoked: false, typeof: null };
      let snakeMethod: MethodResult = { invoked: false, typeof: null };

      try {
        Module.CrabyTestModule.camelMethod();
        camelMethod = { invoked: true, typeof: typeof Module.CrabyTestModule.camelMethod };
      } catch {}

      try {
        Module.CrabyTestModule.PascalMethod();
        pascalMethod = { invoked: true, typeof: typeof Module.CrabyTestModule.PascalMethod };
      } catch {}

      try {
        Module.CrabyTestModule.snake_method();
        snakeMethod = { invoked: true, typeof: typeof Module.CrabyTestModule.snake_method };
      } catch {}

      assert(camelMethod.invoked, '`camelMethod` is not invoked');
      assert(pascalMethod.invoked, '`PascalMethod` is not invoked');
      assert(snakeMethod.invoked, '`snake_method` is not invoked');

      assert(camelMethod.typeof === 'function', '`camelMethod` is not a function');
      assert(pascalMethod.typeof === 'function', '`PascalMethod` is not a function');
      assert(snakeMethod.typeof === 'function', '`snake_method` is not a function');

      return { camelMethod, PascalMethod: pascalMethod, snake_method: snakeMethod };
    },
  },
];

export { TEST_SUITES };
