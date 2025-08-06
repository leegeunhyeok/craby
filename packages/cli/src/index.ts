import { run as runCli } from './cli';
import { logger } from './logger';
import { getBindings } from './napi';

export async function run() {
  const { setup } = getBindings();

  const verbose = Boolean(
    process.argv.find((arg) => arg === '-v' || arg === '--verbose')
  );

  try {
    setup(verbose ? 'debug' : process.env.RUST_LOG);
    runCli();
  } catch (error) {
    logger.error(error instanceof Error ? error.message : 'unknown error');
    process.exit(1);
  }
}
