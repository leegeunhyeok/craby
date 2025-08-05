import type { TurboModule } from 'react-native';
import { TurboModuleRegistry } from 'react-native';

export interface Spec extends TurboModule {
  numericMethod(arg: number): number;
  booleanMethod(arg: boolean): boolean;
  // stringMethod(arg: string): string;
}

export default TurboModuleRegistry.getEnforcing<Spec>('Basic');
