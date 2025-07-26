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
  external: [/\.node$/, '../napi/index.cjs'],
  banner: {
    js: CJS_REQUIRE_SHIM,
  },
  onSuccess: async () => {
    const napiModule = path.resolve('napi/index.js');

    if (fs.existsSync(napiModule)) {
      await fs.promises.rename(
        path.resolve('napi/index.js'),
        path.resolve('napi/index.cjs')
      );
    }
  },
});
