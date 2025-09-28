import { Registry, type Module, type Signal } from 'craby-modules';

export interface TestObject {
  foo: string;
  bar: number;
  baz: boolean;
  sub: SubObject | null;
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

export interface Spec extends Module {
  numericMethod(arg: number): number;
  booleanMethod(arg: boolean): boolean;
  stringMethod(arg: string): string;
  objectMethod(arg: TestObject): TestObject;
  arrayMethod(arg: number[]): number[];
  enumMethod(arg0: MyEnum, arg1: SwitchState): string;
  nullableMethod(arg: number | null): MaybeNumber;
  promiseMethod(arg: number): Promise<number>;
  // Signals
  onSignal: Signal;
  triggerSignal(): void;
}

export default Registry.getEnforcing<Spec>('CrabyTest');
