import path from 'node:path';
import { Command } from '@commander-js/extra-typings';
import { getSchemaInfo } from '../codegen/get-schema-info';
import { isValidProject } from '../utils/is-valid-project';
import { logger } from '../logger';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';

const command = withVerbose(
  new Command().name('init').action(() => {
    const projectRoot = process.cwd();

    if (!isValidProject(projectRoot)) {
      throw new Error('Invalid TurboModule project');
    }

    const schemaInfo = getSchemaInfo(projectRoot);
    logger.debug(`Schema: ${JSON.stringify(schemaInfo, null, 2)}`);

    getBindings().init({
      projectRoot,
      templateBasePath: path.resolve(import.meta.dirname, '..', 'templates'),
      libraryName: schemaInfo.library.name
        .toLowerCase()
        .replace(/[^a-z]/g, '_'),
    });
  })
);

export { command };
