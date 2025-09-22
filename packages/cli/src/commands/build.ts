import { Command } from '@commander-js/extra-typings';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';

const command = withVerbose(
  new Command().name('build').action(() => {
    const projectRoot = process.cwd();
    getBindings().build({ projectRoot });
  })
);

export { command };
