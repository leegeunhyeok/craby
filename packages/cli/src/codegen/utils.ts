/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * @format
 */

import { logger } from '../logger';

function extractLibrariesFromJSON(configFile: any, dependencyPath: string) {
  if (configFile.codegenConfig == null) {
    return [];
  }
  logger.debug(`[Codegen] Found ${configFile.name}`);
  if (configFile.codegenConfig.libraries == null) {
    const config = configFile.codegenConfig;
    return [
      {
        name: configFile.name,
        config,
        libraryPath: dependencyPath,
      },
    ];
  } else {
    printDeprecationWarningIfNeeded(configFile.name);
    return extractLibrariesFromConfigurationArray(configFile, dependencyPath);
  }
}

function extractLibrariesFromConfigurationArray(
  configFile: any,
  dependencyPath: string
) {
  return configFile.codegenConfig.libraries.map((config: any) => {
    return {
      name: config.name,
      config,
      libraryPath: dependencyPath,
    };
  });
}

function printDeprecationWarningIfNeeded(dependency: string) {
  if (dependency === 'react-native') {
    return;
  }
  logger.warn(`CodegenConfig Deprecated Setup for ${dependency}.
    The configuration file still contains the codegen in the libraries array.
    If possible, replace it with a single object.
  `);
  logger.warn(`BEFORE:
    {
      // ...
      "codegenConfig": {
        "libraries": [
          {
            "name": "libName1",
            "type": "all|components|modules",
            "jsSrcsRoot": "libName1/js"
          },
          {
            "name": "libName2",
            "type": "all|components|modules",
            "jsSrcsRoot": "libName2/src"
          }
        ]
      }
    }

    AFTER:
    {
      "codegenConfig": {
        "name": "libraries",
        "type": "all",
        "jsSrcsRoot": "."
      }
    }
  `);
}

export { extractLibrariesFromJSON };
