import path from 'node:path';
import { Command } from '@commander-js/extra-typings';
import { getSchemaInfo } from '../codegen/get-schema-info';
import { isValidProject } from '../utils/is-valid-project';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { assert } from 'es-toolkit';

const command = withVerbose(
  new Command().name('init').action(() => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid TurboModule project');

    getBindings().init({
      projectRoot,
      templateBasePath: path.resolve(import.meta.dirname, '..', 'templates'),
      libraryName: getSchemaInfo(projectRoot).library.name,
    });
  })
);

export { command };
