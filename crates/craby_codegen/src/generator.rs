use craby_common::{
    constants::impl_mod_name,
    utils::string::{pascal_case, snake_case},
};
use indoc::formatdoc;

use crate::{
    platform::{
        cxx::{
            template::{cxx_enum_bridging_template, cxx_struct_bridging_template},
            CxxMethod, ToCxxMethod,
        },
        rust::{RsCxxBridge, ToRsCxxBridge},
    },
    types::{schema::Schema, types::CodegenResult},
    utils::indent_str,
};

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, schema: &Schema) -> Result<CodegenResult, anyhow::Error> {
        let spec_code = self.generate_spec(schema)?;
        let impl_code = self.generate_impl(schema)?;
        let rs_cxx_bridge = self.get_rs_cxx_bridges(schema)?;
        let cxx_methods = self.get_cxx_methods(schema)?;
        let cxx_bridging_templates = self.get_cxx_bridging_templates(schema)?;

        Ok(CodegenResult {
            module_name: schema.module_name.clone(),
            ffi_mod: snake_case(&schema.module_name),
            impl_mod: impl_mod_name(&schema.module_name),
            spec_code,
            impl_code,
            rs_cxx_bridge,
            cxx_methods,
            cxx_bridging_templates,
        })
    }

    /// Generate the spec trait for the given schema.
    ///
    /// ```rust,ignore
    /// pub trait MyModuleSpec {
    ///     fn multiply(a: f64, b: f64) -> f64;
    /// }
    /// ```
    fn generate_spec(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());
        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| -> Result<String, anyhow::Error> {
                let sig = spec.as_sig()?;
                Ok(format!("{};", sig))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // ```rust,ignore
        // pub trait MyModuleSpec {
        //     fn multiply(a: f64, b: f64) -> f64;
        // }
        // ```
        let code = formatdoc! {
          r#"
          pub trait {trait_name} {{
          {methods}
          }}"#,
          trait_name = trait_name,
          methods = indent_str(methods.join("\n"), 4),
        };

        Ok(code)
    }

    /// Generate the empty module for the given schema.
    ///
    /// ```rust,ignore
    /// use crate::{ffi::my_module::*, generated::*};
    ///
    /// pub struct MyModule;
    ///
    /// impl MyModuleSpec for MyModule {
    ///     fn multiply(a: f64, b: f64) -> f64 {
    ///         unimplemented!();
    ///     }
    /// }
    /// ```
    fn generate_impl(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let mod_name = pascal_case(schema.module_name.as_str());
        let snake_name = snake_case(schema.module_name.as_str());
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());

        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| -> Result<String, anyhow::Error> {
                let func_sig = spec.as_sig()?;

                // ```rust,ignore
                // fn multiply(a: f64, b: f64) -> f64 {
                //     unimplemented!();
                // }
                // ```
                let code = formatdoc! {
                  r#"
                  {func_sig} {{
                      unimplemented!();
                  }}"#,
                  func_sig = func_sig,
                };

                Ok(code)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // ```rust,ignore
        // use crate::{ffi::my_module::*, generated::*};
        //
        // pub struct MyModule;
        //
        // impl MyModuleSpec for MyModule {
        //     fn multiply(a: f64, b: f64) -> f64 {
        //         unimplemented!();
        //     }
        // }
        // ```
        let code = formatdoc! {
          r#"
          use crate::{{ffi::{snake_name}::*, generated::*}};

          pub struct {mod_name};

          impl {trait_name} for {mod_name} {{
          {methods}
          }}"#,
          snake_name = snake_name,
          trait_name = trait_name,
          mod_name= mod_name,
          methods = indent_str(methods.join("\n\n"), 4),
        };

        Ok(code)
    }

    /// Returns the cxx function signature for the `FunctionSpec`.
    fn get_rs_cxx_bridges(&self, schema: &Schema) -> Result<RsCxxBridge, anyhow::Error> {
        schema.to_rs_cxx_bridge()
    }

    /// Returns the cxx function implementations for the `FunctionSpec`.
    fn get_cxx_methods(&self, schema: &Schema) -> Result<Vec<CxxMethod>, anyhow::Error> {
        let res = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.to_cxx_method(&schema.module_name))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(res)
    }

    /// Returns the cxx JSI bridging templates for the `Schema`.
    fn get_cxx_bridging_templates(&self, schema: &Schema) -> Result<Vec<String>, anyhow::Error> {
        let mut bridging_templates = vec![];

        schema.alias_map.iter().try_for_each(
            |(name, alias_spec)| -> Result<(), anyhow::Error> {
                let struct_template =
                    cxx_struct_bridging_template(&schema.module_name, name, alias_spec)?;
                bridging_templates.push(struct_template);
                Ok(())
            },
        )?;

        schema
            .enum_map
            .iter()
            .try_for_each(|(name, enum_spec)| -> Result<(), anyhow::Error> {
                let enum_template =
                    cxx_enum_bridging_template(&schema.module_name, name, enum_spec)?;
                bridging_templates.push(enum_template);
                Ok(())
            })?;

        Ok(bridging_templates)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::load_schema_json;

    use super::*;

    #[test]
    fn test_generate_spec() {
        let schema = load_schema_json::<Schema>();
        let generator = CodeGenerator::new();
        let result = generator.generate_spec(&schema).unwrap();

        assert_snapshot!(result);
    }

    #[test]
    fn test_generate_impl() {
        let schema = load_schema_json::<Schema>();
        let generator = CodeGenerator::new();
        let result = generator.generate_impl(&schema).unwrap();

        assert_snapshot!(result);
    }

    #[test]
    fn test_generate_rs_cxx_bridges() {
        let schema = load_schema_json::<Schema>();
        let generator = CodeGenerator::new();
        let result = generator.get_rs_cxx_bridges(&schema).unwrap();

        assert_snapshot!(result.extern_func);
        assert_snapshot!(result.impl_func);
    }

    #[test]
    fn test_get_cxx_bridging_templates() {
        let schema = load_schema_json::<Schema>();
        let generator = CodeGenerator::new();
        let result = generator.get_cxx_bridging_templates(&schema).unwrap();

        assert_snapshot!(result.join("\n"));
    }

    #[test]
    fn test_get_cxx_methods() {
        let schema = load_schema_json::<Schema>();
        let generator = CodeGenerator::new();
        let result = generator.get_cxx_methods(&schema).unwrap();

        assert_snapshot!(result
            .into_iter()
            .map(|method| vec![method.name, method.impl_func, method.metadata])
            .flatten()
            .collect::<Vec<_>>()
            .join("\n"));
    }
}
