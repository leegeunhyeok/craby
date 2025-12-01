import * as Module from 'craby-test';
import { assert, isEqual } from 'es-toolkit';
import type { TestSuite } from './types';
import { createTaskHandler, nextTick, stringToBytes, toErrorObject, waitForSignals } from './utils';

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

      return {
        text,
        type,
        data: Array.from(new Uint8Array(result)),
      };
    },
  },
  {
    label: 'Array',
    action: () => Module.CrabyTestModule.arrayMethod([1, 2, 3]),
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
