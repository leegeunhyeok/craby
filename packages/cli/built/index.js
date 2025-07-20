var __require = /* @__PURE__ */ ((x) => typeof require !== "undefined" ? require : typeof Proxy !== "undefined" ? new Proxy(x, {
  get: (a, b) => (typeof require !== "undefined" ? require : a)[b]
}) : x)(function(x) {
  if (typeof require !== "undefined") return require.apply(this, arguments);
  throw Error('Dynamic require of "' + x + '" is not supported');
});

// src/cli.ts
import { program } from "@commander-js/extra-typings";

// src/commands/init.ts
import path4 from "path";
import { Command } from "@commander-js/extra-typings";
import { assert as assert3 } from "es-toolkit";

// src/codegen/get-schema-info.ts
import fs2 from "fs";
import path2 from "path";
import { assert as assert2 } from "es-toolkit";

// src/napi.ts
import { assert } from "es-toolkit";
var mod = null;
function getBindings() {
  assert(mod, "Bindings not loaded");
  return mod.default;
}
async function loadBindings() {
  return (mod = await import("../napi/index.cjs")).default;
}

// src/logger.ts
var logger = null;
function getLogger() {
  if (logger) {
    return logger;
  }
  const bindings = getBindings();
  return logger = {
    trace: bindings.trace,
    debug: bindings.debug,
    info: bindings.info,
    warn: bindings.warn,
    error: bindings.error
  };
}
var loggerProxy = new Proxy({}, {
  get(_, prop) {
    return (message) => getLogger()[prop](message);
  }
});

// src/codegen/utils.ts
function extractLibrariesFromJSON(configFile, dependencyPath) {
  if (configFile.codegenConfig == null) {
    return [];
  }
  loggerProxy.debug(`[Codegen] Found ${configFile.name}`);
  if (configFile.codegenConfig.libraries == null) {
    const config = configFile.codegenConfig;
    return [
      {
        name: configFile.name,
        config,
        libraryPath: dependencyPath
      }
    ];
  } else {
    printDeprecationWarningIfNeeded(configFile.name);
    return extractLibrariesFromConfigurationArray(configFile, dependencyPath);
  }
}
function extractLibrariesFromConfigurationArray(configFile, dependencyPath) {
  return configFile.codegenConfig.libraries.map((config) => {
    return {
      name: config.name,
      config,
      libraryPath: dependencyPath
    };
  });
}
function printDeprecationWarningIfNeeded(dependency) {
  if (dependency === "react-native") {
    return;
  }
  loggerProxy.warn(`CodegenConfig Deprecated Setup for ${dependency}.
    The configuration file still contains the codegen in the libraries array.
    If possible, replace it with a single object.
  `);
  loggerProxy.warn(`BEFORE:
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

// src/codegen/generate-schema-infos.ts
import fs from "fs";
import path from "path";
import { glob } from "glob";

// src/utils/get-require.ts
import Module from "module";
var ref = typeof __require === "function" ? __require : Module.createRequire(import.meta.url);
function getRequire() {
  return ref;
}

// src/codegen/codegen-utils.ts
function getCombineJSToSchema() {
  let combineJSToSchema;
  const require2 = getRequire();
  try {
    combineJSToSchema = require2("../../packages/react-native-codegen/lib/cli/combine/combine-js-to-schema.js");
  } catch {
    combineJSToSchema = require2("@react-native/codegen/lib/cli/combine/combine-js-to-schema.js");
  }
  if (!combineJSToSchema) {
    throw "combine-js-to-schema not found.";
  }
  return combineJSToSchema;
}

// src/codegen/generate-schema-infos.ts
function generateSchemaInfos(libraries) {
  return libraries.map((library) => generateSchemaInfo(library));
}
function generateSchemaInfo(library, platform) {
  const pathToJavaScriptSources = path.join(
    library.libraryPath,
    library.config.jsSrcsDir
  );
  loggerProxy.debug(`[Codegen] Processing ${library.config.name}`);
  const supportedApplePlatforms = extractSupportedApplePlatforms(
    library.config.name,
    library.libraryPath
  );
  return {
    library,
    supportedApplePlatforms,
    schema: getCombineJSToSchema().combineSchemasInFileList(
      [pathToJavaScriptSources],
      platform,
      /NativeSampleTurboModule/
    )
  };
}
var APPLE_PLATFORMS = ["ios", "macos", "tvos", "visionos"];
function extractSupportedApplePlatforms(dependency, dependencyPath) {
  loggerProxy.debug("[Codegen] Searching for podspec in the project dependencies.");
  const podspecs = glob.sync("*.podspec", { cwd: dependencyPath });
  if (podspecs.length === 0) {
    return;
  }
  const podspec = fs.readFileSync(
    path.join(dependencyPath, podspecs[0]),
    "utf8"
  );
  const supportedPlatforms = podspec.split("\n").filter(
    (line) => line.includes("platform") || line.includes("deployment_target")
  ).join("");
  const supportedPlatformsMap = APPLE_PLATFORMS.reduce(
    (acc, platform) => ({
      ...acc,
      [platform]: supportedPlatforms.includes(
        getCocoaPodsPlatformKey(platform)
      )
    }),
    {}
  );
  const supportedPlatformsList = Object.keys(supportedPlatformsMap).filter(
    (key) => supportedPlatformsMap[key]
  );
  if (supportedPlatformsList.length > 0) {
    loggerProxy.debug(
      `[Codegen] Supported Apple platforms: ${supportedPlatformsList.join(
        ", "
      )} for ${dependency}`
    );
  }
  return supportedPlatformsMap;
}
function getCocoaPodsPlatformKey(platformName) {
  if (platformName === "macos") {
    return "osx";
  }
  return platformName;
}

// src/codegen/get-schema-info.ts
function getSchemaInfo(projectRoot) {
  const config = JSON.parse(
    fs2.readFileSync(path2.join(projectRoot, "package.json"), "utf-8")
  );
  const libraries = extractLibrariesFromJSON(config, projectRoot);
  assert2(libraries.length === 1, "Invalid library config");
  const schemaInfos = generateSchemaInfos(libraries);
  assert2(schemaInfos.length === 1, "Invalid schema info");
  return schemaInfos[0];
}

// src/utils/get-codegen-config.ts
import fs3 from "fs";
import path3 from "path";
function getCodegenConfig(projectRoot) {
  const packageJsonPath = path3.join(projectRoot, "package.json");
  const rawPackageJson = fs3.readFileSync(packageJsonPath, "utf8");
  const packageJson = JSON.parse(rawPackageJson);
  return packageJson.codegenConfig ?? null;
}

// src/utils/is-valid-project.ts
function isValidProject(projectRoot) {
  try {
    return isValidProjectImpl(projectRoot);
  } catch {
    return false;
  }
}
function isValidProjectImpl(projectRoot) {
  return Boolean(getCodegenConfig(projectRoot));
}

// src/utils/with-verbose.ts
import { Option } from "commander";
var VERBOSE_OPTION = new Option("-v, --verbose", "Print all logs");
function withVerbose(command7) {
  return command7.addOption(VERBOSE_OPTION);
}

// src/commands/init.ts
var command = withVerbose(
  new Command().name("init").action(() => {
    const projectRoot = process.cwd();
    assert3(isValidProject(projectRoot), "Invalid TurboModule project");
    getBindings().init({
      projectRoot,
      templateBasePath: path4.resolve(import.meta.dirname, "..", "templates"),
      libraryName: getSchemaInfo(projectRoot).library.name
    });
  })
);

// src/commands/codegen.ts
import { Command as Command2 } from "@commander-js/extra-typings";
import { assert as assert4 } from "es-toolkit";
var command2 = withVerbose(
  new Command2().name("codegen").action(() => {
    const projectRoot = process.cwd();
    assert4(isValidProject(projectRoot), "Invalid TurboModule project");
    const codegenConfig = getCodegenConfig(projectRoot);
    assert4(
      codegenConfig,
      "`codegenConfig` field not found in the `package.json`"
    );
    assert4(
      codegenConfig.android?.javaPackageName,
      "`codegenConfig.android.javaPackageName` is required"
    );
    const schemaInfo = getSchemaInfo(projectRoot);
    loggerProxy.debug(`Schema: ${JSON.stringify(schemaInfo, null, 2)}`);
    const modules = schemaInfo.schema?.modules ?? {};
    const moduleNames = Object.keys(modules);
    if (moduleNames.length === 0) {
      loggerProxy.info("Nothing to generate");
      return;
    }
    getBindings().codegen({
      projectRoot,
      libraryName: schemaInfo.library.name,
      javaPackageName: codegenConfig.android.javaPackageName,
      schemas: moduleNames.map((name) => JSON.stringify(modules[name]))
    });
  })
);

// src/commands/build.ts
import { Command as Command3 } from "@commander-js/extra-typings";
import { assert as assert5 } from "es-toolkit";
var command3 = withVerbose(
  new Command3().name("build").action(() => {
    const projectRoot = process.cwd();
    assert5(isValidProject(projectRoot), "Invalid TurboModule project");
    getBindings().build({
      projectRoot,
      libraryName: getSchemaInfo(projectRoot).library.name
    });
  })
);

// src/commands/show.ts
import { Command as Command4 } from "@commander-js/extra-typings";
import { assert as assert6 } from "es-toolkit";
var command4 = withVerbose(
  new Command4().name("show").action(() => {
    const projectRoot = process.cwd();
    assert6(isValidProject(projectRoot), "Invalid TurboModule project");
    const schemaInfo = getSchemaInfo(projectRoot);
    loggerProxy.debug(`Schema: ${JSON.stringify(schemaInfo, null, 2)}`);
    const modules = schemaInfo.schema?.modules ?? {};
    const moduleNames = Object.keys(modules);
    if (moduleNames.length === 0) {
      loggerProxy.info("Nothing to show");
      return;
    }
    getBindings().show({
      projectRoot,
      libraryName: getSchemaInfo(projectRoot).library.name,
      schemas: moduleNames.map((name) => JSON.stringify(modules[name]))
    });
  })
);

// src/commands/doctor.ts
import { Command as Command5 } from "@commander-js/extra-typings";
var command5 = withVerbose(
  new Command5().name("doctor").action(() => {
    const projectRoot = process.cwd();
    getBindings().doctor({ projectRoot });
  })
);

// src/commands/clean.ts
import { Command as Command6 } from "@commander-js/extra-typings";
import { assert as assert7 } from "es-toolkit";
var command6 = withVerbose(
  new Command6().name("clean").action(() => {
    const projectRoot = process.cwd();
    assert7(isValidProject(projectRoot), "Invalid TurboModule project");
    getBindings().clean({ projectRoot });
  })
);

// package.json
var version = "0.1.0-alpha.0";

// src/cli.ts
function run() {
  const cli = program.name("craby").version(version);
  cli.addCommand(command);
  cli.addCommand(command2);
  cli.addCommand(command3);
  cli.addCommand(command4);
  cli.addCommand(command5);
  cli.addCommand(command6);
  cli.parse();
}

// src/index.ts
async function run2() {
  const { setup } = await loadBindings();
  const verbose = Boolean(
    process.argv.find((arg) => arg === "-v" || arg === "--verbose")
  );
  try {
    setup(verbose ? "debug" : process.env.RUST_LOG);
    run();
  } catch (error) {
    loggerProxy.error(error instanceof Error ? error.message : "unknown error");
    process.exit(1);
  }
}
export {
  run2 as run
};
