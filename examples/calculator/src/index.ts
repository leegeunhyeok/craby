import Calculator from './NativeCalculator';

export function add(a: number, b: number): number {
  return Calculator.add(a, b);
}

export function subtract(a: number, b: number): number {
  return Calculator.subtract(a, b);
}

export function multiply(a: number, b: number): number {
  return Calculator.multiply(a, b);
}

export function divide(a: number, b: number): number {
  return Calculator.divide(a, b);
}
