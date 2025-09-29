import { Command } from '@commander-js/extra-typings';
import { getBindings } from '@craby/cli-bindings';
import { withVerbose } from 'src/utils/command';
import { resolveProjectRoot } from 'src/utils/resolve-project-root';

export async function runCodegen() {
  getBindings().codegen({ projectRoot: resolveProjectRoot() });
}

export const command = withVerbose(new Command().name('codegen').action(runCodegen));
