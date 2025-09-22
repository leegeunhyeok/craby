import { Command } from '@commander-js/extra-typings';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';

const command = withVerbose(
  new Command().name('codegen').action(async () => {
    const projectRoot = process.cwd();
    getBindings().codegen({ projectRoot });
  })
);

export { command };
