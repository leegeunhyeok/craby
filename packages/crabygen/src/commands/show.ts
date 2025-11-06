import { Command } from '@commander-js/extra-typings';
import { show } from '@craby/cli-bindings';
import { withVerbose } from '../utils/command';
import { withErrorHandler } from '../utils/errors';

export const command = withVerbose(
  new Command().name('show').action(withErrorHandler(show.bind(null, { projectRoot: process.cwd() }))),
);
