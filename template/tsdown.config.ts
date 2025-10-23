import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: './src/index.ts',
  outDir: './dist',
  format: ['esm', 'cjs'],
  sourcemap: true,
  dts: true,
});
