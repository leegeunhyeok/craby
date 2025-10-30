use std::{fs, path::PathBuf};

use craby_common::{constants::ios_base_path, utils::string::flat_case};
use indoc::formatdoc;

use crate::{
    constants::{cxx_mod_cls_name, objc_mod_provider_name},
    types::CodegenContext,
    utils::indent_str,
};

use super::types::{GenerateResult, Generator, GeneratorInvoker, Template};

pub struct IosTemplate;
pub struct IosGenerator;

pub enum IosFileType {
    ModuleProvider,
}

impl IosTemplate {
    /// Generates the iOS module provider implementation.
    ///
    /// # Generated Code
    ///
    /// ```objc
    /// #import "CxxMyTestModule.hpp"
    /// #import <ReactCommon/CxxTurboModuleUtils.h>
    /// #include <string>
    ///
    /// @interface CrabyMyAppModuleProvider : NSObject
    /// @end
    ///
    /// @implementation CrabyMyAppModuleProvider
    ///
    /// + (void)load {
    ///   const char *cDataPath = [[self getDataPath] UTF8String];
    ///   std::string dataPath(cDataPath);
    ///
    ///   craby::mymodule::CxxMyTestModule::dataPath = dataPath;
    ///
    ///   facebook::react::registerCxxModuleToGlobalModuleMap(
    ///       craby::mymodule::CxxMyTestModule::kModuleName,
    ///       [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {
    ///         return std::make_shared<craby::mymodule::CxxMyTestModule>(jsInvoker);
    ///       });
    /// }
    ///
    /// + (NSString *)getDataPath {
    ///   NSString *appGroupID = [[NSBundle mainBundle] objectForInfoDictionaryKey:@"AppGroupID"];
    ///   NSString *dataPath = nil;
    ///
    ///   if (appGroupID != nil) {
    ///     NSFileManager *fileManager = [NSFileManager defaultManager];
    ///     NSURL *containerURL = [fileManager containerURLForSecurityApplicationGroupIdentifier:appGroupID];
    ///
    ///     if (containerURL == nil) {
    ///       throw [NSException exceptionWithName:@"CrabyInitializationException"
    ///                                     reason:[NSString stringWithFormat:@"Invalid AppGroup ID: %@", appGroupID]
    ///                                   userInfo:nil];
    ///     } else {
    ///       dataPath = [containerURL path];
    ///     }
    ///   } else {
    ///     NSArray *paths = NSSearchPathForDirectoriesInDomains(NSDocumentDirectory, NSUserDomainMask, true);
    ///     dataPath = [paths firstObject];
    ///   }
    ///
    ///   return dataPath;
    /// }
    ///
    /// @end
    /// ```
    fn module_provider(&self, ctx: &CodegenContext) -> Result<String, anyhow::Error> {
        let mut cxx_includes = vec![];
        let mut cxx_prepares = Vec::with_capacity(ctx.schemas.len());
        let mut cxx_registers = Vec::with_capacity(ctx.schemas.len());
        let objc_mod_provider_name = objc_mod_provider_name(&ctx.name);

        ctx.schemas.iter().for_each(|schema| {
            let flat_name = flat_case(&schema.module_name);
            let cxx_mod = cxx_mod_cls_name(&schema.module_name);
            let cxx_namespace = format!("craby::{}::{}", flat_name, cxx_mod);
            let cxx_include = format!("#import \"{cxx_mod}.hpp\"");
            let cxx_prepare = format!("{cxx_namespace}::dataPath = dataPath;");
            let cxx_register = formatdoc! {
                r#"
                facebook::react::registerCxxModuleToGlobalModuleMap(
                    {cxx_namespace}::kModuleName,
                    [](std::shared_ptr<facebook::react::CallInvoker> jsInvoker) {{
                      return std::make_shared<{cxx_namespace}>(jsInvoker);
                    }});"#,
                cxx_namespace = cxx_namespace,
            };

            cxx_includes.push(cxx_include);
            cxx_prepares.push(cxx_prepare);
            cxx_registers.push(cxx_register);
        });

        let content = formatdoc! {
            r#"
            {cxx_includes}
            #import <ReactCommon/CxxTurboModuleUtils.h>
            #include <string>

            @interface {objc_mod_provider_name} : NSObject
            @end

            @implementation {objc_mod_provider_name}

            + (void)load {{
              const char *cDataPath = [[self getDataPath] UTF8String];
              std::string dataPath(cDataPath);

            {cxx_prepares}

            {cxx_registers}
            }}

            + (NSString *)getDataPath {{
              NSString *appGroupID = [[NSBundle mainBundle] objectForInfoDictionaryKey:@"AppGroupID"];
              NSString *dataPath = nil;

              if (appGroupID != nil) {{
                NSFileManager *fileManager = [NSFileManager defaultManager];
                NSURL *containerURL = [fileManager containerURLForSecurityApplicationGroupIdentifier:appGroupID];

                if (containerURL == nil) {{
                  throw [NSException exceptionWithName:@"CrabyInitializationException"
                                                reason:[NSString stringWithFormat:@"Invalid AppGroup ID: %@", appGroupID]
                                              userInfo:nil];
                  }} else {{
                    dataPath = [containerURL path];
                  }}
              }} else {{
                NSArray *paths = NSSearchPathForDirectoriesInDomains(NSDocumentDirectory, NSUserDomainMask, true);
                dataPath = [paths firstObject];
              }}

              return dataPath;
            }}

            @end"#,
            cxx_includes = cxx_includes.join("\n"),
            cxx_prepares = indent_str(&cxx_prepares.join("\n"), 2),
            cxx_registers = indent_str(&cxx_registers.join("\n"), 2),
            objc_mod_provider_name = objc_mod_provider_name,
        };

        Ok(content)
    }
}

impl Template for IosTemplate {
    type FileType = IosFileType;

    fn render(
        &self,
        ctx: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let res = match file_type {
            IosFileType::ModuleProvider => {
                vec![(
                    PathBuf::from(format!("{}.mm", objc_mod_provider_name(&ctx.name))),
                    self.module_provider(ctx)?,
                )]
            }
        };

        Ok(res)
    }
}

impl Default for IosGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IosGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator<IosTemplate> for IosGenerator {
    fn cleanup(ctx: &CodegenContext) -> Result<(), anyhow::Error> {
        let src_path = ios_base_path(&ctx.root).join("src");

        if src_path.try_exists()? {
            fs::read_dir(src_path)?.try_for_each(|entry| -> Result<(), anyhow::Error> {
                let path = entry?.path();
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                if file_name.ends_with(".mm") {
                    fs::remove_file(&path)?;
                }

                Ok(())
            })?;
        }

        Ok(())
    }

    fn generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let ios_base_path = ios_base_path(&ctx.root);
        let template = self.template_ref();
        let mut files = vec![];

        let provider_res = template
            .render(ctx, &IosFileType::ModuleProvider)?
            .into_iter()
            .map(|(path, content)| GenerateResult {
                path: ios_base_path.join(path),
                content,
                overwrite: true,
            })
            .collect::<Vec<_>>();

        files.extend(provider_res);

        Ok(files)
    }

    fn template_ref(&self) -> &IosTemplate {
        &IosTemplate
    }
}

impl GeneratorInvoker for IosGenerator {
    fn invoke_generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        self.generate(ctx)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::get_codegen_context;

    use super::*;

    #[test]
    fn test_ios_generator() {
        let ctx = get_codegen_context();
        let generator = IosGenerator::new();
        let results = generator.generate(&ctx).unwrap();
        let result = results
            .iter()
            .map(|res| format!("{}\n{}", res.path.display(), res.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        assert_snapshot!(result);
    }
}
