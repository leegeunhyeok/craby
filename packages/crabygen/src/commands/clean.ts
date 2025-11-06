import { Command } from '@commander-js/extra-typings';
import { clean } from '@craby/cli-bindings';
import { withVerbose } from '../utils/command';
import { withErrorHandler } from '../utils/errors';

export const command = withVerbose(
  new Command().name('clean').action(withErrorHandler(clean.bind(null, { projectRoot: process.cwd() }))),
);
