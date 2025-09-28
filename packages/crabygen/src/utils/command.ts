import { Command, Option } from '@commander-js/extra-typings';
import { getBindings } from '@craby/cli-bindings';
import { getCommonOptions } from './common-options';

type BindingMethod = keyof ReturnType<typeof getBindings>;

const VERBOSE_OPTION = new Option('-v, --verbose', 'Print all logs');

function withVerbose<T extends Command>(command: T) {
  return command.addOption(VERBOSE_OPTION);
}

export function createBindingCommand(
  commandName: Extract<BindingMethod, 'init' | 'codegen' | 'build' | 'show' | 'doctor' | 'clean'>,
) {
  const command = new Command().name(commandName).action(async () => {
    const execute = getBindings()[commandName];
    execute(getCommonOptions());
  });

  return withVerbose(command);
}
