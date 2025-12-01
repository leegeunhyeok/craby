import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: './src/index.ts',
  outDir: './dist',
  format: ['esm', 'cjs'],
  fixedExtension: false,
  dts: true,
});
