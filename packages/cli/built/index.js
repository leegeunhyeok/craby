import $$__Module from 'node:module';
typeof require !== 'function' && (globalThis.require = $$__Module.createRequire(import.meta.url));

// src/cli.ts
import { program } from "@commander-js/extra-typings";

// src/utils/command.ts
import { Command, Option } from "@commander-js/extra-typings";

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

// src/napi.ts
import * as mod from "../napi/index.js";
function getBindings() {
  return mod;
}

// src/utils/command.ts
var VERBOSE_OPTION = new Option("-v, --verbose", "Print all logs");
function withVerbose(command7) {
  return command7.addOption(VERBOSE_OPTION);
}
function createBindingCommand(commandName) {
  const command7 = new Command().name(commandName).action(async () => {
    const execute = getBindings()[commandName];
    execute(getCommonOptions());
  });
  return withVerbose(command7);
}

// src/commands/init.ts
var command = createBindingCommand("init");

// src/commands/codegen.ts
var command2 = createBindingCommand("codegen");

// src/commands/build.ts
var command3 = createBindingCommand("build");

// src/commands/show.ts
var command4 = createBindingCommand("show");

// src/commands/doctor.ts
var command5 = createBindingCommand("doctor");

// src/commands/clean.ts
var command6 = createBindingCommand("clean");

// package.json
var version = "0.1.0-alpha.3";

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

// src/index.ts
async function run2() {
  const { setup } = getBindings();
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
