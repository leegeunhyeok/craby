import { Command } from '@commander-js/extra-typings';
import { init } from '@craby/cli-bindings';
import { withVerbose } from '../utils/command';
import { withErrorHandler } from '../utils/errors';

export const command = withVerbose(
  new Command()
    .name('init')
    .argument('<packageName>', 'The name of the package')
    .action((packageName) => withErrorHandler(init.bind(null, { cwd: process.cwd(), pkgName: packageName }))()),
);
