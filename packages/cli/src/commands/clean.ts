import { Command } from '@commander-js/extra-typings';
import { assert } from 'es-toolkit';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { isValidProject } from '../utils/is-valid-project';

const command = withVerbose(
  new Command().name('clean').action(() => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid TurboModule project');

    getBindings().clean({ projectRoot });
  })
);

export { command };
