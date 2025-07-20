import { program } from '@commander-js/extra-typings';
import { command as initCommand } from './commands/init';
import { command as codegenCommand } from './commands/codegen';
import { command as buildCommand } from './commands/build';
import { command as showCommand } from './commands/show';
import { command as doctorCommand } from './commands/doctor';
import { command as cleanCommand } from './commands/clean';
import { version } from '../package.json';

export function run() {
  const cli = program.name('craby').version(version);

  cli.addCommand(initCommand);
  cli.addCommand(codegenCommand);
  cli.addCommand(buildCommand);
  cli.addCommand(showCommand);
  cli.addCommand(doctorCommand);
  cli.addCommand(cleanCommand);

  cli.parse();
}
