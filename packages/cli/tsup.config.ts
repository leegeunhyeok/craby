import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from 'tsup';

const CJS_REQUIRE_SHIM = `
import $$__Module from 'node:module';
typeof require !== 'function' && (globalThis.require = $$__Module.createRequire(import.meta.url));
`.trim();

export default defineConfig({
  entry: ['src/index.ts'],
  outDir: './built',
  format: 'esm',
  platform: 'node',
  target: 'node20',
  sourcemap: false,
  dts: false,
  shims: true,
  clean: true,
  external: [/\.node$/, '../napi/index.js'],
  banner: {
    js: CJS_REQUIRE_SHIM,
  },
});
