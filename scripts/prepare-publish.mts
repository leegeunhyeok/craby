import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { isValidVersion } from './utils.mjs';
import { type PackageInfo, Yarn } from './yarn.mjs';

async function updateVersion(packageInfo: PackageInfo, version: string) {
  const rawPackageJson = await readFile(join(packageInfo.location, 'package.json'), 'utf8');
  const packageJson = JSON.parse(rawPackageJson);
  packageJson.version = version;
  await writeFile(join(packageInfo.location, 'package.json'), JSON.stringify(packageJson, null, 2));
}

const version = process.argv[2];

if (version == null) {
  console.error('Version is required');
  process.exit(1);
} else if (!isValidVersion(version)) {
  console.error('Invalid version', version);
  process.exit(1);
}

console.log(`Updating version to ${version}`);
for (const packageInfo of await Yarn.collectPackages()) {
  await updateVersion(packageInfo, version);
}

console.log(
  `
To publish, commit changes and push:

git add -A
git commit -m "${version}"
`.trim(),
);
