import { TurboModuleRegistry } from 'react-native';

interface Module {}

interface Signal {
  (handler: () => void): () => void;
}

interface Registry {
  get<T extends Module>(moduleName: string): T | null;
  getEnforcing<T extends Module>(moduleName: string): T;
}

export const Registry: Registry = {
  get<T extends Module>(moduleName: string): T | null {
    return TurboModuleRegistry.get<T>(moduleName);
  },
  getEnforcing<T extends Module>(moduleName: string): T {
    return TurboModuleRegistry.getEnforcing<T>(moduleName);
  },
};

export type { Module, Signal };
