import path from 'node:path';
import { Command } from '@commander-js/extra-typings';
import { getSchemaInfo } from '../codegen/get-schema-info';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { logger } from 'src/logger';

const command = withVerbose(
  new Command().name('init').action(async () => {
    const projectRoot = process.cwd();
    const schemaInfo = await getSchemaInfo(projectRoot);
    const modules = schemaInfo.schema?.modules ?? {};
    const moduleNames = Object.keys(modules);

    if (moduleNames.length === 0) {
      logger.error('TurboModule schema is not found');
      return;
    }

    getBindings().init({
      projectRoot,
      templateBasePath: path.resolve(import.meta.dirname, '..', 'templates'),
      packageName: schemaInfo.library.name,
      schemas: moduleNames.map((name) => JSON.stringify(modules[name])),
    });
  })
);

export { command };
