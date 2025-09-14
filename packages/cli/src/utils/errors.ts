import { logger } from "src/logger";

export function withErrorHandler<T>(task: Promise<T>) {
  task.catch((error) => {
    logger.error(error.message);
  });

  return task;
}
