import Module from 'node:module';

let ref =
  typeof require === 'function'
    ? require
    : Module.createRequire(import.meta.url);

export function getRequire() {
  return ref;
}
