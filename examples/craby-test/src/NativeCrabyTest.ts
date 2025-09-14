import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface TestObject {
  foo: string;
  bar: number;
  baz: boolean;
}

export enum MyEnum {
  FOO = 'FOO',
  BAR = 'BAR',
  BAZ = 'BAZ',
}

export type Direction = 'up' | 'down' | 'left' | 'right';

export interface Spec extends TurboModule {
  numericMethod(arg: number): number;
  booleanMethod(arg: boolean): boolean;
  stringMethod(arg: string): string;
  objectMethod(arg: TestObject): TestObject;
  arrayMethod(arg: number[]): number[];
  enumMethod(arg: MyEnum): string;
  unionMethod(arg: Direction): string;
}

export default TurboModuleRegistry.getEnforcing<Spec>('CrabyTest');
