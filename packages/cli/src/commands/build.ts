import { Command } from '@commander-js/extra-typings';
import { assert } from 'es-toolkit';
import { getSchemaInfo } from '../codegen/get-schema-info';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { isValidProject } from '../utils/is-valid-project';

const command = withVerbose(
  new Command().name('build').action(() => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid TurboModule project');

    getBindings().build({
      projectRoot,
      libraryName: getSchemaInfo(projectRoot).library.name,
    });
  })
);

export { command };
