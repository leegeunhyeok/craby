import { error, setup } from '@craby/cli-bindings';
import { run as runCli } from './cli';

export async function run(baseCommand = 'crabygen') {
  const verbose = Boolean(process.argv.find((arg) => arg === '-v' || arg === '--verbose'));

  try {
    setup(verbose ? 'debug' : process.env.RUST_LOG);
    runCli(baseCommand);
  } catch (reason) {
    error(reason instanceof Error ? reason.message : 'unknown error');
    process.exit(1);
  }
}
