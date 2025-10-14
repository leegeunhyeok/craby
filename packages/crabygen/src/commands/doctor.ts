import { Command } from '@commander-js/extra-typings';
import { getBindings } from '../utils/bindings';
import { withVerbose } from '../utils/command';
import { commonErrorHandler } from '../utils/errors';
import { resolveProjectRoot } from '../utils/resolve-project-root';

export const command = withVerbose(
  new Command().name('doctor').action(async () => {
    try {
      getBindings().doctor({ projectRoot: resolveProjectRoot() });
    } catch (error) {
      commonErrorHandler(error);
    }
  }),
);
