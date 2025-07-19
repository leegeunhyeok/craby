import { getCodegenConfig } from './get-codegen-config';

export function isValidProject(projectRoot: string) {
  try {
    return isValidProjectImpl(projectRoot);
  } catch {
    return false;
  }
}

function isValidProjectImpl(projectRoot: string) {
  return Boolean(getCodegenConfig(projectRoot));
}
