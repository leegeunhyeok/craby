import { logger } from '../logger';

export function commonErrorHandler(error: unknown) {
  if (error instanceof Error) {
    logger.error(error.message);
  } else {
    logger.error('Unknown error');
  }
  process.exit(1);
}
