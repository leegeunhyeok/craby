import CrabyTestModule, { type TestObject } from './NativeCrabyTest';

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

export type { TestObject };
