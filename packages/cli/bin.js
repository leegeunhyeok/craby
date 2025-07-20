#!/usr/bin/env node

await import('./built/index.js')
  .then(({ run }) => run())
  .catch((error) => {
    console.error('Unexpected error:', error);
    process.exit(1);
  });
