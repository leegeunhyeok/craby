import { Command } from '@commander-js/extra-typings';
import { codegen } from '@craby/cli-bindings';
import { withVerbose } from '../utils/command';
import { withErrorHandler } from '../utils/errors';

export const runCodegen = withErrorHandler(codegen.bind(null, { projectRoot: process.cwd() }));

export const command = withVerbose(new Command().name('codegen').action(runCodegen));
