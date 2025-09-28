import * as mod from '../napi/index.js';

export type BindingMethod = keyof typeof mod;

export function getBindings() {
  return mod;
}
