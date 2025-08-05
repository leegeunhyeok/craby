import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface Spec extends TurboModule {
  numericMethod(arg: number): number;
  stringMethod(arg: string): string;
  booleanMethod(arg: boolean): boolean;
}

export default TurboModuleRegistry.getEnforcing<Spec>('Basic');
