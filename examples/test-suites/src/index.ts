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
    label: 'Array',
    action: () => Module.arrayMethod([1, 2, 3]),
  },
  {
    label: 'Enum',
    action: () => Module.enumMethod(Module.MyEnum.FOO),
  },
  {
    label: 'Enum',
    description: '(Invalid enum value)',
    action: () => {
      try {
        return Module.enumMethod('UNKNOWN' as any)
      } catch (error: any) {
        return toErrorObject(error);
      }
    },
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
];

export { TEST_SUITES };
