import { Command } from '@commander-js/extra-typings';
import { getBindings } from '@craby/cli-bindings';
import { withVerbose } from 'src/utils/command';

export const command = withVerbose(
  new Command()
    .name('init')
    .argument('<packageName>', 'The name of the package')
    .action(async (packageName) => {
      getBindings().init({ cwd: process.cwd(), pkgName: packageName });
    }),
);
