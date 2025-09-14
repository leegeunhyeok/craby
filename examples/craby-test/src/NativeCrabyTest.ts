import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface TestObject {
  foo: string;
  bar: number;
  baz: boolean;
}

export interface Spec extends TurboModule {
  numericMethod(arg: number): number;
  booleanMethod(arg: boolean): boolean;
  stringMethod(arg: string): string;
  objectMethod(arg: TestObject): TestObject;
  arrayMethod(arg: number[]): number[];
}

export default TurboModuleRegistry.getEnforcing<Spec>('CrabyTest');
