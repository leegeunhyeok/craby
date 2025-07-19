import type { Command } from '@commander-js/extra-typings';
import { Option } from 'commander';

const VERBOSE_OPTION = new Option('-v, --verbose', 'Print all logs');

export function withVerbose<T extends Command>(command: T) {
  return command.addOption(VERBOSE_OPTION);
}
