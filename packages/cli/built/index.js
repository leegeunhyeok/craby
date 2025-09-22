import $$__Module from 'node:module';
typeof require !== 'function' && (globalThis.require = $$__Module.createRequire(import.meta.url));

// src/cli.ts
import { program } from "@commander-js/extra-typings";

// src/commands/init.ts
import path2 from "path";
import { Command } from "@commander-js/extra-typings";

// src/napi.ts
import * as mod from "../napi/index.js";
function getBindings() {
  return mod;
}

// src/utils/with-verbose.ts
import { Option } from "commander";
var VERBOSE_OPTION = new Option("-v, --verbose", "Print all logs");
function withVerbose(command7) {
  return command7.addOption(VERBOSE_OPTION);
}

// src/utils/package-json.ts
import fs from "fs";
import path from "path";
function getPackageJsonPath(projectRoot) {
  return path.join(projectRoot, "package.json");
}
function getPackageJson(projectRoot) {
  return JSON.parse(fs.readFileSync(getPackageJsonPath(projectRoot), "utf8"));
}

// src/commands/init.ts
var command = withVerbose(
  new Command().name("init").action(async () => {
    const projectRoot = process.cwd();
    const packageJson = getPackageJson(projectRoot);
    getBindings().init({
      projectRoot,
      templateBasePath: path2.resolve(import.meta.dirname, "..", "templates"),
      packageName: packageJson.name
    });
  })
);

// src/commands/codegen.ts
import { Command as Command2 } from "@commander-js/extra-typings";
var command2 = withVerbose(
  new Command2().name("codegen").action(async () => {
    const projectRoot = process.cwd();
    getBindings().codegen({ projectRoot });
  })
);

// src/commands/build.ts
import { Command as Command3 } from "@commander-js/extra-typings";
var command3 = withVerbose(
  new Command3().name("build").action(() => {
    const projectRoot = process.cwd();
    getBindings().build({ projectRoot });
  })
);

// src/commands/show.ts
import { Command as Command4 } from "@commander-js/extra-typings";
var command4 = withVerbose(
  new Command4().name("show").action(async () => {
    const projectRoot = process.cwd();
    getBindings().show({ projectRoot });
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
var command6 = withVerbose(
  new Command6().name("clean").action(() => {
    const projectRoot = process.cwd();
    getBindings().clean({ projectRoot });
  })
);

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
