import { getWorkspaceRoot } from '../utils';
import { type CreateResolverOptions, createResolver } from './resolver';

interface GetMetroConfigOptions {
  resolverOptions?: Omit<CreateResolverOptions, 'rootPath'>;
}

type MetroConfig = any;

export function getMetroConfig(rootDir: string, options: GetMetroConfigOptions) {
  return (previousConfig: MetroConfig) => {
    return {
      ...previousConfig,
      projectRoot: rootDir,
      watchFolders: [...(previousConfig.watchFolders ?? []), getWorkspaceRoot(rootDir)],
      resolver: {
        ...previousConfig.resolver,
        resolveRequest: createResolver({ rootPath: rootDir, ...options?.resolverOptions }),
      },
    };
  };
}
