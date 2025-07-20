import { Command } from '@commander-js/extra-typings';
import { assert } from 'es-toolkit';
import { isValidProject } from '../utils/is-valid-project';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';

const command = withVerbose(
  new Command().name('doctor').action(() => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid TurboModule project');

    getBindings().doctor({ projectRoot });
  })
);

export { command };
