import { program } from '@commander-js/extra-typings';
import { version } from '../package.json';
import { command as buildCommand } from './commands/build';
import { command as cleanCommand } from './commands/clean';
import { command as codegenCommand } from './commands/codegen';
import { command as doctorCommand } from './commands/doctor';
import { command as initCommand } from './commands/init';
import { command as showCommand } from './commands/show';

export function run(baseCommand: string) {
  const cli = program.name(baseCommand).version(version);

  cli.addCommand(codegenCommand);
  cli.addCommand(initCommand);
  cli.addCommand(buildCommand);
  cli.addCommand(showCommand);
  cli.addCommand(doctorCommand);
  cli.addCommand(cleanCommand);

  cli.parse(
    isCodegenCommand(process.argv)
      ? [process.argv[0], process.argv[1], 'codegen', ...process.argv.slice(2)]
      : process.argv,
  );
}

function isCodegenCommand(argv: string[]) {
  const options = argv.slice(2);
  return options.every((option) => option.startsWith('-'));
}
