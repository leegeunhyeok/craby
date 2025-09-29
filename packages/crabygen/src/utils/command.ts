import { type Command, Option } from '@commander-js/extra-typings';

const VERBOSE_OPTION = new Option('-v, --verbose', 'Print all logs');

export function withVerbose<T extends Command<any[], {}, {}>>(command: T) {
  return command.addOption(VERBOSE_OPTION);
}
