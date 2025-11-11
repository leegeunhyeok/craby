import { Command } from '@commander-js/extra-typings';
import { codegen } from '@craby/cli-bindings';
import { withVerbose } from '../utils/command';
import { withErrorHandler } from '../utils/errors';

export const runCodegen = withErrorHandler((overwrite: boolean) => codegen({ projectRoot: process.cwd(), overwrite }));

export const command = withVerbose(
  new Command()
    .name('codegen')
    .option('--no-overwrite', 'Do not overwrite existing files')
    .action((options) => runCodegen(options.overwrite)),
);
