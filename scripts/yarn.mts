import { $ } from 'zx';

export interface PackageInfo {
  location: string;
  name: string;
}

async function collectPackages() {
  const result = await $`yarn workspaces list --json`;
  return result.stdout
    .split('\n')
    .filter((line) => line.includes('packages/'))
    .map((line) => JSON.parse(line) as PackageInfo);
}

export const Yarn = {
  collectPackages,
};
