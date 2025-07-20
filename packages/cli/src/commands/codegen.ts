import { Command } from '@commander-js/extra-typings';
import { assert } from 'es-toolkit';
import { getSchemaInfo } from '../codegen/get-schema-info';
import { logger } from '../logger';
import { getBindings } from '../napi';
import { withVerbose } from '../utils/with-verbose';
import { getCodegenConfig } from '../utils/get-codegen-config';
import { isValidProject } from '../utils/is-valid-project';

const command = withVerbose(
  new Command().name('codegen').action(() => {
    const projectRoot = process.cwd();
    assert(isValidProject(projectRoot), 'Invalid TurboModule project');

    const codegenConfig = getCodegenConfig(projectRoot);
    assert(
      codegenConfig,
      '`codegenConfig` field not found in the `package.json`'
    );
    assert(
      codegenConfig.android?.javaPackageName,
      '`codegenConfig.android.javaPackageName` is required'
    );

    const schemaInfo = getSchemaInfo(projectRoot);
    logger.debug(`Schema: ${JSON.stringify(schemaInfo, null, 2)}`);

    const modules = schemaInfo.schema?.modules ?? {};
    const moduleNames = Object.keys(modules);

    if (moduleNames.length === 0) {
      logger.info('Nothing to generate');
      return;
    }

    getBindings().codegen({
      projectRoot,
      libraryName: schemaInfo.library.name,
      javaPackageName: codegenConfig.android.javaPackageName,
      schemas: moduleNames.map((name) => JSON.stringify(modules[name])),
    });
  })
);

export { command };
