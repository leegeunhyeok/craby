import type { NativeModule, Signal } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

export interface TestObject {
  foo: string;
  bar: number;
  baz: boolean;
  sub: SubObject | null;
  camelCase: number;
  PascalCase: number;
  snake_case: number;
}

export type SubObject = {
  a: string | null;
  b: number;
  c: boolean;
};

export type MaybeNumber = number | null;

export enum MyEnum {
  Foo = 'foo',
  Bar = 'bar',
  Baz = 'baz',
}

export enum SwitchState {
  Off = 0,
  On = 1,
}

export interface Spec extends NativeModule {
  numericMethod(arg: number): number;
  booleanMethod(arg: boolean): boolean;
  stringMethod(arg: string): string;
  objectMethod(arg: TestObject): TestObject;
  arrayMethod(arg: number[]): number[];
  enumMethod(arg0: MyEnum, arg1: SwitchState): string;
  nullableMethod(arg: number | null): MaybeNumber;
  promiseMethod(arg: number): Promise<number>;
  setState(arg: number): void;
  getState(): number;
  // Naming conventions
  camelMethod(): void;
  PascalMethod(): void;
  snake_method(): void;
  // Signals
  onSignal: Signal;
  triggerSignal(): void;
}

export default NativeModuleRegistry.getEnforcing<Spec>('CrabyTest');
