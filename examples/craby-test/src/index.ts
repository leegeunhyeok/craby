import CrabyTestModule, {
  MyEnum,
  type TestObject,
  type SubObject,
} from './NativeCrabyTest';

export function numericMethod(arg: number) {
  return CrabyTestModule.numericMethod(arg);
}

export function booleanMethod(arg: boolean) {
  return CrabyTestModule.booleanMethod(arg);
}

export function stringMethod(arg: string) {
  return CrabyTestModule.stringMethod(arg);
}

export function objectMethod(arg: TestObject) {
  return CrabyTestModule.objectMethod(arg);
}

export function arrayMethod(arg: number[]) {
  return CrabyTestModule.arrayMethod(arg);
}

export function enumMethod(arg: MyEnum) {
  return CrabyTestModule.enumMethod(arg);
}

export function nullableMethod(arg: number | null) {
  return CrabyTestModule.nullableMethod(arg);
}

export function promiseMethod(arg: number) {
  return CrabyTestModule.promiseMethod(arg);
}

export type { TestObject, SubObject };
export { MyEnum };
