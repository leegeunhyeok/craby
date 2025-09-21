use std::path::PathBuf;

use craby_common::{constants::ios_base_path, utils::string::flat_case};
use indoc::formatdoc;

use crate::{
    constants::{cxx_mod_cls_name, objc_mod_provider_name},
    types::schema::Schema,
    utils::indent_str,
};

use super::types::{GenerateResult, Generator, GeneratorInvoker, Template};

pub struct IosTemplate;
pub struct IosGenerator;

pub enum IosFileType {
    ModuleProvider,
}

impl IosTemplate {
    fn module_provider(
        &self,
        project_name: &String,
        schemas: &Vec<Schema>,
    ) -> Result<String, anyhow::Error> {
        let mut cxx_includes = vec![];
        let mut cxx_registers = vec![];

        // TODO: support multiple schemas
        let objc_mod_provider_name = objc_mod_provider_name(project_name);

        schemas.iter().for_each(|schema| {
            let flat_name = flat_case(&schema.module_name);
            let cxx_mod = cxx_mod_cls_name(&schema.module_name);
            let cxx_namespace = format!("craby::{}::{}", flat_name, cxx_mod);
            let cxx_include = format!("#import \"{cxx_mod}.hpp\"");
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
            cxx_registers.push(cxx_register);
        });

        let content = formatdoc! {
            r#"
            {cxx_includes}

            #import <ReactCommon/CxxTurboModuleUtils.h>

            @interface {objc_mod_provider_name} : NSObject
            @end

            @implementation {objc_mod_provider_name}
            + (void)load {{
            {cxx_registers}
            }}
            @end"#,
            cxx_includes = cxx_includes.join("\n"),
            cxx_registers = indent_str(cxx_registers.join("\n"), 2),
            objc_mod_provider_name = objc_mod_provider_name,
        };

        Ok(content)
    }
}

impl Template for IosTemplate {
    type FileType = IosFileType;

    fn render(
        &self,
        schemas: &Vec<Schema>,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        // TODO: support multiple schemas
        let project_name = schemas.get(0).unwrap().module_name.clone();
        let res = match file_type {
            IosFileType::ModuleProvider => {
                vec![(
                    PathBuf::from(format!("{}.mm", objc_mod_provider_name(&project_name))),
                    self.module_provider(&project_name, schemas)?,
                )]
            }
        };

        Ok(res)
    }
}

impl IosGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator<IosTemplate> for IosGenerator {
    fn generate(
        &self,
        project_root: &PathBuf,
        schemas: &Vec<Schema>,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let ios_base_path = ios_base_path(project_root);
        let template = self.template_ref();
        let mut files = vec![];

        let provider_res = template
            .render(schemas, &IosFileType::ModuleProvider)?
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
    fn invoke_generate(
        &self,
        project_root: &PathBuf,
        schemas: &Vec<Schema>,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        self.generate(project_root, schemas)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::load_schema_json;

    use super::*;

    #[test]
    fn test_ios_generator() {
        let schema = load_schema_json::<Schema>();
        let generator = IosGenerator::new();
        let results = generator
            .generate(&PathBuf::from("."), &vec![schema])
            .unwrap();

        assert_snapshot!(results
            .iter()
            .map(|res| format!("{}\n{}", res.path.display(), res.content))
            .collect::<Vec<_>>()
            .join("\n\n"));
    }
}
