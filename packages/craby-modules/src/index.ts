import { TurboModuleRegistry } from 'react-native';

type NativeModule = {};

type Signal = (handler: () => void) => () => void;

interface NativeModuleRegistry {
  get<T extends NativeModule>(moduleName: string): T | null;
  getEnforcing<T extends NativeModule>(moduleName: string): T;
}

export const NativeModuleRegistry: NativeModuleRegistry = {
  get<T extends NativeModule>(moduleName: string): T | null {
    return TurboModuleRegistry.get<T>(moduleName);
  },
  getEnforcing<T extends NativeModule>(moduleName: string): T {
    return TurboModuleRegistry.getEnforcing<T>(moduleName);
  },
};

export type { NativeModule, Signal };
