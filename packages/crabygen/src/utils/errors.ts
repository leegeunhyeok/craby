import { error } from '@craby/cli-bindings';

export function commonErrorHandler(reason: unknown) {
  if (reason instanceof Error) {
    error(reason.message);
  } else {
    error('Unknown error');
  }
  process.exit(1);
}

export function withErrorHandler(action: () => void) {
  return () => {
    try {
      action();
    } catch (reason) {
      commonErrorHandler(reason);
    }
  };
}
