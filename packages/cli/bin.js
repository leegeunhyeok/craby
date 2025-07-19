#!/usr/bin/env node

await import('./dist/index.js')
  .then(({ run }) => run())
  .catch((error) => {
    console.error('Unexpected error:', error);
    process.exit(1);
  });
