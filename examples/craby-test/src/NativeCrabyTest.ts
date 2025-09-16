import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface TestObject {
  foo: string;
  bar: number;
  baz: boolean;
  sub: SubObject;
}

export interface SubObject {
  a?: string;
  b: number;
  c: boolean;
}

export enum MyEnum {
  FOO = 'FOO',
  BAR = 'BAR',
  BAZ = 'BAZ',
}

export interface Spec extends TurboModule {
  numericMethod(arg: number): number;
  booleanMethod(arg: boolean): boolean;
  stringMethod(arg: string): string;
  objectMethod(arg: TestObject): TestObject;
  arrayMethod(arg: number[]): number[];
  enumMethod(arg: MyEnum): string;
  promiseMethod(arg: number): Promise<number>;
}

export default TurboModuleRegistry.getEnforcing<Spec>('CrabyTest');
