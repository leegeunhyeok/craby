import Basic from './NativeBasic';

export function numeric(arg: number): number {
  return Basic.numericMethod(arg);
}

export function boolean(arg: boolean): boolean {
  return Basic.booleanMethod(arg);
}

export function string(arg: string): string {
  return arg; // TODO
}
