import { program } from '@commander-js/extra-typings';
import { version } from '../package.json';
import { createBindingCommand } from './utils/command';

export function run() {
  const cli = program.name('crabygen').version(version);

  cli.addCommand(createBindingCommand('init'));
  cli.addCommand(createBindingCommand('codegen'));
  cli.addCommand(createBindingCommand('build'));
  cli.addCommand(createBindingCommand('show'));
  cli.addCommand(createBindingCommand('doctor'));
  cli.addCommand(createBindingCommand('clean'));

  cli.parse();
}
