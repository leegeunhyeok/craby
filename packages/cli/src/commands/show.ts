import { Command } from '@commander-js/extra-typings';
import { assert } from 'es-toolkit';
import { getSchemaInfo } from '../codegen/get-schema-info';
import { isValidProject } from '../utils/is-valid-project';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { logger } from '../logger';

const command = withVerbose(
  new Command().name('show').action(() => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid TurboModule project');

    const schemaInfo = getSchemaInfo(projectRoot);
    logger.debug(`Schema: ${JSON.stringify(schemaInfo, null, 2)}`);

    const modules = schemaInfo.schema?.modules ?? {};
    const moduleNames = Object.keys(modules);

    if (moduleNames.length === 0) {
      logger.info('Nothing to show');
      return;
    }

    getBindings().show({
      projectRoot,
      libraryName: getSchemaInfo(projectRoot).library.name,
      schemas: moduleNames.map((name) => JSON.stringify(modules[name])),
    });
  })
);

export { command };
