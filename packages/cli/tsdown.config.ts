import fs from 'node:fs/promises';
import path from 'node:path';
import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: 'src/index.ts',
  outDir: './dist',
  format: 'esm',
  target: 'node20',
  sourcemap: false,
  dts: false,
  shims: true,
  external: [/\.node$/],
  onSuccess: async () => {
    const files = await fs.readdir(path.resolve('napi'));
    const nodeFiles = files.filter((file) => file.endsWith('.node'));

    for (const file of nodeFiles) {
      console.log('Copying binary file:', file);
      await fs.copyFile(path.resolve('napi', file), path.resolve('dist', file));
    }
  },
});
