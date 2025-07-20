import { assert } from 'es-toolkit';

type Mod = typeof import('../napi/index');
let mod: (Mod & { default: Mod }) | null = null;

export function getBindings() {
  assert(mod, 'Bindings not loaded');
  return mod.default as Mod;
}

export async function loadBindings() {
  return (mod = await import('../napi/index')).default;
}
