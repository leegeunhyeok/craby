import { Command } from '@commander-js/extra-typings';
import { getBindings } from '../utils/bindings';
import { withVerbose } from '../utils/command';
import { commonErrorHandler } from '../utils/errors';

export const command = withVerbose(
  new Command()
    .name('init')
    .argument('<packageName>', 'The name of the package')
    .action(async (packageName) => {
      try {
        getBindings().init({ cwd: process.cwd(), pkgName: packageName });
      } catch (error) {
        commonErrorHandler(error);
      }
    }),
);
