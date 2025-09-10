import { Command } from '@commander-js/extra-typings';
import { assert } from 'es-toolkit';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { isValidProject } from '../utils/is-valid-project';

const command = withVerbose(
  new Command().name('build').action(() => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid Craby project');

    getBindings().build({ projectRoot });
  })
);

export { command };
