import { Command } from '@commander-js/extra-typings';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';

const command = withVerbose(
  new Command().name('clean').action(() => {
    const projectRoot = process.cwd();
    getBindings().clean({ projectRoot });
  })
);

export { command };
