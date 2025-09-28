import { Registry, type Module } from 'craby-modules';

export interface Spec extends Module {
  add(a: number, b: number): number;
  subtract(a: number, b: number): number;
  multiply(a: number, b: number): number;
  divide(a: number, b: number): number;
}

export default Registry.getEnforcing<Spec>('Calculator');
