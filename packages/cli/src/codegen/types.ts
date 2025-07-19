export interface Library {
  name: string;
  config: { name: string; type: string; jsSrcsDir: string };
  libraryPath: string;
}

export interface SchemaInfo {
  library: Library;
  supportedApplePlatforms: {
    ios: boolean;
    macos: boolean;
    tvos: boolean;
    visionos: boolean;
  };
  schema: {
    modules: Record<
      string,
      {
        moduleName: string;
        type: string;
        aliasMap: Record<string, string>;
        enumMap: Record<string, string>;
        spec: {
          eventEmitters: any[];
          methods: {
            name: string;
            optional: boolean;
            typeAnnotation: {
              type: string;
              returnTypeAnnotation: {
                type: string;
              };
              params: {
                name: string;
                optional: boolean;
                typeAnnotation: {
                  type: string;
                };
              }[];
            };
          }[];
        };
      }
    >;
  };
}
