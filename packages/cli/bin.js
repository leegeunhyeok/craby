#!/usr/bin/env node

import { run } from './built/index.js';

run().catch((error) => {
  console.error('Unexpected error:', error);
  process.exit(1);
});
