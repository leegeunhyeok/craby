import { Command } from '@commander-js/extra-typings';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';

const command = withVerbose(
  new Command().name('doctor').action(() => {
    const projectRoot = process.cwd();

    getBindings().doctor({ projectRoot });
  })
);

export { command };
