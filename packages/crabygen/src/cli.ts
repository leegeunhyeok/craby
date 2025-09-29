import { program } from '@commander-js/extra-typings';
import { version } from '../package.json';
import { command as buildCommand } from './commands/build';
import { command as cleanCommand } from './commands/clean';
import { command as codegenCommand, runCodegen } from './commands/codegen';
import { command as doctorCommand } from './commands/doctor';
import { command as initCommand } from './commands/init';
import { command as showCommand } from './commands/show';

export function run() {
  const cli = program.name('crabygen').version(version);

  // Codegen is the default command
  cli.action(runCodegen);
  cli.addCommand(codegenCommand);

  // Other commands
  cli.addCommand(initCommand);
  cli.addCommand(buildCommand);
  cli.addCommand(showCommand);
  cli.addCommand(doctorCommand);
  cli.addCommand(cleanCommand);

  cli.parse();
}
