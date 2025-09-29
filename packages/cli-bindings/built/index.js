import $$__Module from 'node:module';
typeof require !== 'function' && (globalThis.require = $$__Module.createRequire(import.meta.url));

// src/napi.ts
import * as mod from "../napi/index.js";
function getBindings() {
  return mod;
}
export {
  getBindings
};
