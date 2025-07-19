import fs from 'node:fs';
import path from 'node:path';
import { assert } from 'es-toolkit';
import { extractLibrariesFromJSON } from './utils';
import { generateSchemaInfos } from './generate-schema-infos';

export function getSchemaInfo(projectRoot: string) {
  const config = JSON.parse(
    fs.readFileSync(path.join(projectRoot, 'package.json'), 'utf-8')
  );

  const libraries = extractLibrariesFromJSON(config, projectRoot);
  assert(libraries.length === 1, 'Invalid library config');

  const schemaInfos = generateSchemaInfos(libraries);
  assert(schemaInfos.length === 1, 'Invalid schema info');

  return schemaInfos[0]!;
}
