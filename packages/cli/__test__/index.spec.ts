import { describe, it, expect } from 'vitest';
import * as mod from '../napi/index.js';

describe('bindings', () => {
  describe('common', () => {
    it('`setup` should be a function', () => {
      expect(typeof mod.setup).toBe('function');
    });

    it('`trace` should be a function', () => {
      expect(typeof mod.trace).toBe('function');
    });

    it('`debug` should be a function', () => {
      expect(typeof mod.debug).toBe('function');
    });

    it('`info` should be a function', () => {
      expect(typeof mod.info).toBe('function');
    });

    it('`warn` should be a function', () => {
      expect(typeof mod.warn).toBe('function');
    });

    it('`error` should be a function', () => {
      expect(typeof mod.error).toBe('function');
    });
  });

  describe('commands', () => {
    it('`init` should be a function', () => {
      expect(typeof mod.init).toBe('function');
    });

    it('`codegen` should be a function', () => {
      expect(typeof mod.codegen).toBe('function');
    });

    it('`build` should be a function', () => {
      expect(typeof mod.build).toBe('function');
    });

    it('`show` should be a function', () => {
      expect(typeof mod.show).toBe('function');
    });

    it('`doctor` should be a function', () => {
      expect(typeof mod.doctor).toBe('function');
    });

    it('`clean` should be a function', () => {
      expect(typeof mod.clean).toBe('function');
    });
  });
});
