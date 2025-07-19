import { getBindings } from './napi';

type Logger = {
  trace: (message: string) => void;
  debug: (message: string) => void;
  info: (message: string) => void;
  warn: (message: string) => void;
  error: (message: string) => void;
};

let logger: Logger | null = null;

function getLogger() {
  if (logger) {
    return logger;
  }

  const bindings = getBindings();

  return (logger = {
    trace: bindings.trace,
    debug: bindings.debug,
    info: bindings.info,
    warn: bindings.warn,
    error: bindings.error,
  });
}

const loggerProxy = new Proxy({} as Logger, {
  get(_, prop) {
    return (message: string) => getLogger()[prop as keyof Logger](message);
  },
});

export { loggerProxy as logger };
