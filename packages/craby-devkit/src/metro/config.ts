import { getWorkspaceRoot } from '../utils';
import { type CreateResolverOptions, createResolver } from './resolver';

interface GetMetroConfigOptions {
  resolverOptions?: Omit<CreateResolverOptions, 'rootPath'>;
}

export function getMetroConfig(rootDir: string, options: GetMetroConfigOptions) {
  return {
    projectRoot: rootDir,
    watchFolders: [getWorkspaceRoot(rootDir)],
    resolver: {
      resolveRequest: createResolver({ rootPath: rootDir, ...options?.resolverOptions }),
    },
  };
}
