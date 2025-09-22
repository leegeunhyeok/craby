import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface TestObject {
  foo: string;
  bar: number;
  baz: boolean;
  sub: SubObject | null;
}

export interface SubObject {
  a: string | null;
  b: number;
  c: boolean;
}

export enum MyEnum {
  Foo = 'foo',
  Bar = 'bar',
  Baz = 'baz',
}

export enum SwitchState {
  Off = 0,
  On = 1,
}

export interface Spec extends TurboModule {
  numericMethod(arg: number): number;
  booleanMethod(arg: boolean): boolean;
  stringMethod(arg: string): string;
  objectMethod(arg: TestObject): TestObject;
  arrayMethod(arg: number[]): number[];
  enumMethod(arg0: MyEnum, arg1: SwitchState): string;
  nullableMethod(arg: number | null): number | null;
  promiseMethod(arg: number): Promise<number>;
}

export default TurboModuleRegistry.getEnforcing<Spec>('CrabyTest');
