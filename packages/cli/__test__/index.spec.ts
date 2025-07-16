import { describe, it, expect } from 'vitest';
import { run } from '../index.js';

describe('run from native', () => {
  it('should be a function', () => {
    expect(typeof run).toBe('function');
  });
});
