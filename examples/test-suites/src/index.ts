import * as Module from 'craby-test';
import type { TestSuite } from './types';
import { toErrorObject } from './utils';

const TEST_SUITES: TestSuite[] = [
  {
    label: 'Number',
    action: () => Module.numericMethod(123),
  },
  {
    label: 'Boolean',
    action: () => Module.booleanMethod(true),
  },
  {
    label: 'String',
    action: () => Module.stringMethod('Hello, World!'),
  },
  {
    label: 'Object',
    action: () => Module.objectMethod({
      foo: 'foo',
      bar: 123,
      baz: false,
      sub: {
        a: 'a',
        b: 456,
        c: true,
      },
    }),
  },
  {
    label: 'Object',
    description: '(Invalid object)',
    action: () => {
      try {
        return Module.objectMethod({ foo: 123 } as any);
      } catch (error: any) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Object',
    description: '(Nullable 1)',
    action: () => {
      try {
        return Module.objectMethod({
          foo: 'foo',
          bar: 456,
          baz: true,
          sub: null,
        });
      } catch (error: any) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Object',
    description: '(Nullable 2)',
    action: () => {
      try {
        return Module.objectMethod({
          foo: 'foo',
          bar: 456,
          baz: true,
          sub: {
            a: null,
            b: 789,
            c: false,
          },
        });
      } catch (error: any) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Array',
    action: () => Module.arrayMethod([1, 2, 3]),
  },
  {
    label: 'Enum',
    action: () => Module.enumMethod(Module.MyEnum.Foo, Module.SwitchState.Off),
  },
  {
    label: 'Enum',
    description: '(Invalid string enum value)',
    action: () => {
      try {
        return Module.enumMethod('UNKNOWN' as any, Module.SwitchState.Off);
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
        return Module.enumMethod(Module.MyEnum.Baz, -999 as any);
      } catch (error: any) {
        return toErrorObject(error);
      }
    },
  },
  {
    label: 'Nullable',
    description: '(Non null)',
    action: () => Module.nullableMethod(123),
  },
  {
    label: 'Nullable',
    description: '(Null -> Non null)',
    action: () => Module.nullableMethod(null),
  },
  {
    label: 'Nullable',
    description: '(Non null -> Null)',
    action: () => Module.nullableMethod(-123),
  },
  {
    label: 'Promise',
    action: () => Module.promiseMethod(123),
  },
  {
    label: 'Promise',
    description: '(Rejected promise)',
    action: () => Module.promiseMethod(-123).catch((error) => toErrorObject(error)),
  },
  {
    label: 'Multiple TurboModules',
    description: 'Calculator',
    action: () => {
      const a = 5;
      const b = 10;

      return {
        add: Module.add(a, b),
        subtract: Module.subtract(a, b),
        multiply: Module.multiply(a, b),
        divide: Module.divide(a, b),
      };
    },
  }
];

export { TEST_SUITES };
