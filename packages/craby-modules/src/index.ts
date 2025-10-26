import { Platform, TurboModuleRegistry } from 'react-native';

type NativeModule = {};

type Signal = (handler: () => void) => () => void;

/**
 * Android JNI initialization workaround
 *
 * We need `filesDir` of `Context` for JNI initialization, but it's unavailable during `PackageList` construction.
 * The context is only passed when React Native calls `BaseReactPackage.getModule()`.
 *
 * Workaround: Load a dummy module to trigger `getModule()` before the actual module.
 *
 * - 1. Request non-existent module → triggers `getModule()`
 * - 2. `getModule()` receives `ReactApplicationContext`
 *   - 2-1. Calls `nativeSetDataPath()` (C++ extern function) to set `context.filesDir.absolutePath`
 *   - 2-2. Returns placeholder module (no-op) instance (Actual C++ TurboModule is now can be initialized with the required values)
 *
 * @param moduleName The name of the module to prepare.
 */
function prepareJNI(moduleName: string) {
  if (Platform.OS !== 'android') {
    return;
  }

  TurboModuleRegistry.get(`__craby${moduleName}_JNI_prepare__`);
}

interface NativeModuleRegistry {
  get<T extends NativeModule>(moduleName: string): T | null;
  getEnforcing<T extends NativeModule>(moduleName: string): T;
}

export const NativeModuleRegistry: NativeModuleRegistry = {
  get<T extends NativeModule>(moduleName: string): T | null {
    prepareJNI(moduleName);
    return TurboModuleRegistry.get<T>(moduleName);
  },
  getEnforcing<T extends NativeModule>(moduleName: string): T {
    prepareJNI(moduleName);
    return TurboModuleRegistry.getEnforcing<T>(moduleName);
  },
};

export type { NativeModule, Signal };
