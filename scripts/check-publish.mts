import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { $ } from 'zx';
import { isValidVersion } from './utils.mjs';
import { type PackageInfo, Yarn } from './yarn.mjs';

async function getReleaseVersion() {
  const result = await $`git log -1 --pretty=%B`;
  const version = result.stdout.trim();

  return isValidVersion(version) ? version : null;
}

async function checkVersion(packageInfos: PackageInfo[]) {
  for (const packageInfo of packageInfos) {
    const rawPackageJson = await readFile(join(packageInfo.location, 'package.json'), 'utf8');
    const packageJson = JSON.parse(rawPackageJson);

    if (packageJson.version !== releaseVersion) {
      throw new Error(`Version mismatch for ${packageInfo.name}: ${packageJson.version} !== ${releaseVersion}`);
    }
  }
}

const releaseVersion = await getReleaseVersion();

if (releaseVersion == null) {
  console.log('Not a release, skipping version check');
  process.exit(0);
}

const packages = await Yarn.collectPackages();

console.log(`Release version: ${releaseVersion}`);
console.log(`Checking version for ${packages.length} packages...`);

await checkVersion(packages).catch((error) => {
  console.error(error);
  process.exit(1);
});
