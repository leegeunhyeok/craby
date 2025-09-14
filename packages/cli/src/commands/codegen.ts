import { Command } from '@commander-js/extra-typings';
import { assert } from 'es-toolkit';
import { getSchemaInfo } from '../codegen/get-schema-info';
import { logger } from '../logger';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { isValidProject } from '../utils/is-valid-project';

const command = withVerbose(
  new Command().name('codegen').action(async () => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid TurboModule project');

    const schemaInfo = await getSchemaInfo(projectRoot);
    logger.debug(`Schema: ${JSON.stringify(schemaInfo, null, 2)}`);

    const modules = schemaInfo.schema?.modules ?? {};
    const moduleNames = Object.keys(modules);

    if (moduleNames.length === 0) {
      logger.error('TurboModule schema is not found');
      return;
    }

    const schemas = moduleNames.map((name) => JSON.stringify(modules[name]));
    logger.debug(`Schemas: ${schemas.join('\n')}`);

    getBindings().codegen({
      projectRoot,
      schemas,
    });
  })
);

export { command };
