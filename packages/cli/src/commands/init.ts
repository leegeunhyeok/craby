import path from 'node:path';
import { Command } from '@commander-js/extra-typings';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { getPackageJson } from 'src/utils/package-json';

const command = withVerbose(
  new Command().name('init').action(async () => {
    const projectRoot = process.cwd();
    const packageJson = getPackageJson(projectRoot);

    getBindings().init({
      projectRoot,
      templateBasePath: path.resolve(import.meta.dirname, '..', 'templates'),
      packageName: packageJson.name,
    });
  })
);

export { command };
