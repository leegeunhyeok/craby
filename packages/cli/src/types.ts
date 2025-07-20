export type Config = {
  name: string;
  type: 'modules' | 'components' | 'all';
  jsSrcsDir: string;
  android?: {
    javaPackageName?: string;
  };
  includesGeneratedCode?: boolean;
};
