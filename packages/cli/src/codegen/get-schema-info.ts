import { assert } from 'es-toolkit';
import { generateSchemaInfos } from './generate-schema-infos';
import { getPackageJson } from 'src/utils/package-json';

export async function getSchemaInfo(projectRoot: string) {
  const packageJson = getPackageJson(projectRoot);
  const schemaInfos = generateSchemaInfos([{
    name: packageJson.name,
    config: {
      name: packageJson.name,
      type: 'modules',
      jsSrcsDir: 'src',
    },
    libraryPath: projectRoot,
  }]);
  assert(schemaInfos.length === 1, 'Invalid schema info');

  return schemaInfos[0]!;
}
