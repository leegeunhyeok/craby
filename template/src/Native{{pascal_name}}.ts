import type { NativeModule } from 'craby-modules';
import { NativeModuleRegistry } from 'craby-modules';

interface Spec extends NativeModule {
  add(a: number, b: number): number;
  subtract(a: number, b: number): number;
  multiply(a: number, b: number): number;
  divide(a: number, b: number): number;
}

export default NativeModuleRegistry.getEnforcing<Spec>('{{ pascal_name }}');
