import $$__Module from 'node:module';
typeof require !== 'function' && (globalThis.require = $$__Module.createRequire(import.meta.url));

// src/cli.ts
import { program } from "@commander-js/extra-typings";

// package.json
var version = "0.1.0-alpha.3";

// src/utils/command.ts
import { Command, Option } from "@commander-js/extra-typings";

// src/napi.ts
import * as mod from "../napi/index.js";
function getBindings() {
  return mod;
}

// src/utils/common-options.ts
import path2 from "path";

// src/utils/package-json.ts
import fs from "fs";
import path from "path";
function getPackageJsonPath(projectRoot) {
  return path.join(projectRoot, "package.json");
}
function getPackageJson(projectRoot) {
  return JSON.parse(fs.readFileSync(getPackageJsonPath(projectRoot), "utf8"));
}

// src/utils/common-options.ts
function getCommonOptions() {
  const projectRoot = process.cwd();
  const packageJson = getPackageJson(projectRoot);
  return {
    projectRoot,
    templateBasePath: path2.resolve(import.meta.dirname, "..", "templates"),
    packageName: packageJson.name
  };
}

// src/utils/command.ts
var VERBOSE_OPTION = new Option("-v, --verbose", "Print all logs");
function withVerbose(command) {
  return command.addOption(VERBOSE_OPTION);
}
function createBindingCommand(commandName) {
  const command = new Command().name(commandName).action(async () => {
    const execute = getBindings()[commandName];
    execute(getCommonOptions());
  });
  return withVerbose(command);
}

// src/cli.ts
function run() {
  const cli = program.name("craby").version(version);
  cli.addCommand(createBindingCommand("init"));
  cli.addCommand(createBindingCommand("codegen"));
  cli.addCommand(createBindingCommand("build"));
  cli.addCommand(createBindingCommand("show"));
  cli.addCommand(createBindingCommand("doctor"));
  cli.addCommand(createBindingCommand("clean"));
  cli.parse();
}

// src/logger.ts
var logger = null;
function getLogger() {
  if (logger) {
    return logger;
  }
  const bindings = getBindings();
  logger = {
    trace: bindings.trace,
    debug: bindings.debug,
    info: bindings.info,
    warn: bindings.warn,
    error: bindings.error
  };
  return logger;
}
var loggerProxy = new Proxy({}, {
  get(_, prop) {
    return (message) => getLogger()[prop](message);
  }
});

// src/index.ts
async function run2() {
  const { setup } = getBindings();
  const verbose = Boolean(process.argv.find((arg) => arg === "-v" || arg === "--verbose"));
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
