import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  outDir: './built',
  format: 'esm',
  target: 'node20',
  sourcemap: false,
  dts: false,
  shims: true,
  clean: true,
  external: [/\.node$/, '../napi/index.cjs'],
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
