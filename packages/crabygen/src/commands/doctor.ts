import { Command } from '@commander-js/extra-typings';
import { doctor } from '@craby/cli-bindings';
import { withVerbose } from '../utils/command';
import { withErrorHandler } from '../utils/errors';

export const command = withVerbose(
  new Command().name('doctor').action(withErrorHandler(doctor.bind(null, { projectRoot: process.cwd() }))),
);
