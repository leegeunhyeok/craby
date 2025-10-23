import path from 'node:path';
import { ResolverFactory } from 'oxc-resolver';

export interface CreateResolverOptions {
  rootPath: string;
  rootModules?: string[];
}

/**
 * @see https://github.com/facebook/metro/blob/v0.83.3/packages/metro-resolver/types/types.d.ts#L195-L199
 */
export type CustomResolver = (
  context: CustomResolutionContext,
  moduleName: string,
  platform: string | null,
) => Resolution;

export type SourceFileResolution = Readonly<{
  type: 'sourceFile';
  filePath: string;
}>;

export type AssetFileResolution = ReadonlyArray<string>;
export type AssetResolution = Readonly<{
  type: 'assetFiles';
  filePaths: AssetFileResolution;
}>;
export type Resolution = AssetResolution | SourceFileResolution;

export interface CustomResolutionContext {
  sourceExts: string[];
  originModulePath: string;
  preferNativePlatform?: boolean;
}

const DEFAULT_ROOT_MODULES = ['react', 'react-native'];

const resolvers = new Map();

export function createResolver(options: CreateResolverOptions): CustomResolver {
  function createResolverImpl(context: CustomResolutionContext, platform: string | null, rootPath: string) {
    const rootModules = options.rootModules ?? DEFAULT_ROOT_MODULES;
    const baseExtensions = context.sourceExts.map((extension) => `.${extension}`);
    let finalExtensions = [...baseExtensions];

    if (context.preferNativePlatform) {
      finalExtensions = [...baseExtensions.map((extension) => `.native${extension}`), ...finalExtensions];
    }

    if (platform) {
      finalExtensions = [...baseExtensions.map((extension) => `.${platform}${extension}`), ...finalExtensions];
    }

    const resolver = new ResolverFactory({
      extensions: finalExtensions,
      conditionNames: ['react-native', 'require', 'node', 'default'],
      mainFields: ['react-native', 'browser', 'main'],
      mainFiles: ['index'],
      modules: ['node_modules', path.join(rootPath, 'src')],
    });

    function resolveSync(resolveDir: string, request: string) {
      const resolved = resolver.sync(resolveDir, request);

      if (resolved.path == null) {
        throw new Error(`Failed to resolve '${request}' from '${resolveDir}'`);
      }

      return resolved.path;
    }

    function resolve(context: CustomResolutionContext, request: string) {
      for (const target of rootModules) {
        if (request === target || request.startsWith(`${target}/`)) {
          return {
            type: 'sourceFile',
            filePath: resolveSync(rootPath, request),
          };
        }
      }

      return {
        type: 'sourceFile',
        filePath: resolveSync(path.dirname(context.originModulePath), request),
      };
    }

    return resolve;
  }

  return function resolve(context: CustomResolutionContext, request: string, platform: string | null) {
    let resolver = resolvers.get(platform);

    if (resolver == null) {
      resolver = createResolverImpl(context, platform, options.rootPath);
      resolvers.set(platform, resolver);
    }

    return resolver(context, request, platform);
  };
}
