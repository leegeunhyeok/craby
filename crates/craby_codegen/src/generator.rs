use std::collections::BTreeMap;

use craby_common::{
    constants::impl_mod_name,
    utils::string::{flat_case, pascal_case, snake_case},
};
use indoc::formatdoc;

use crate::{
    platform::{
        cxx::{
            template::{
                cxx_enum_bridging_template, cxx_nullable_bridging_template,
                cxx_struct_bridging_template,
            },
            CxxMethod,
        },
        rust::RsCxxBridge,
    },
    types::{
        schema::{Schema, TypeAnnotation},
        types::CodegenResult,
    },
    utils::{calc_deps_order, indent_str},
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
        let rs_type_impls = self.get_rs_type_impls(schema)?;
        let cxx_methods = self.get_cxx_methods(schema)?;
        let cxx_bridging_templates = self.get_cxx_bridging_templates(schema)?;

        Ok(CodegenResult {
            module_name: schema.module_name.clone(),
            impl_mod: impl_mod_name(&schema.module_name),
            spec_code,
            impl_code,
            rs_cxx_bridge,
            rs_type_impls,
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
                let sig = spec.as_impl_sig()?;
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
                let func_sig = spec.as_impl_sig()?;

                // ```rust,ignore
                // fn multiply(a: Number, b: Number) -> Number {
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
        schema.as_rs_cxx_bridge()
    }

    fn get_rs_type_impls(
        &self,
        schema: &Schema,
    ) -> Result<BTreeMap<String, String>, anyhow::Error> {
        schema.as_rs_type_impls()
    }

    /// Returns the cxx function implementations for the `FunctionSpec`.
    fn get_cxx_methods(&self, schema: &Schema) -> Result<Vec<CxxMethod>, anyhow::Error> {
        let res = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.as_cxx_method(&schema.module_name))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(res)
    }

    /// Returns the cxx JSI bridging templates for the `Schema`.
    fn get_cxx_bridging_templates(&self, schema: &Schema) -> Result<Vec<String>, anyhow::Error> {
        let mut bridging_templates = BTreeMap::new();
        let mut enum_bridging_templates = BTreeMap::new();
        let mut nullable_bridging_templates = BTreeMap::new();

        schema
            .spec
            .methods
            .iter()
            .try_for_each(|spec| -> Result<(), anyhow::Error> {
                if let TypeAnnotation::FunctionTypeAnnotation {
                    params,
                    return_type_annotation,
                } = &*spec.type_annotation
                {
                    params
                        .iter()
                        .try_for_each(|param| -> Result<(), anyhow::Error> {
                            if let nullable_type @ TypeAnnotation::NullableTypeAnnotation {
                                type_annotation,
                            } = &*param.type_annotation
                            {
                                let key = nullable_type.as_cxx_type(&schema.module_name)?;

                                if nullable_bridging_templates.contains_key(&key) {
                                    return Ok(());
                                }

                                let bridging_template = cxx_nullable_bridging_template(
                                    &schema.module_name,
                                    &nullable_type.as_cxx_type(&schema.module_name)?,
                                    type_annotation,
                                )?;

                                nullable_bridging_templates.insert(key, bridging_template);
                            }

                            Ok(())
                        })?;

                    if let nullable_type @ TypeAnnotation::NullableTypeAnnotation {
                        type_annotation,
                    } = &**return_type_annotation
                    {
                        let key = nullable_type.as_cxx_type(&schema.module_name)?;

                        if nullable_bridging_templates.contains_key(&key) {
                            return Ok(());
                        }

                        let bridging_template = cxx_nullable_bridging_template(
                            &schema.module_name,
                            &nullable_type.as_cxx_type(&schema.module_name)?,
                            type_annotation,
                        )?;

                        nullable_bridging_templates.insert(key, bridging_template);
                    }
                }

                Ok(())
            })?;

        schema.alias_map.iter().try_for_each(
            |(name, alias_spec)| -> Result<(), anyhow::Error> {
                let template = cxx_struct_bridging_template(&schema.module_name, name, alias_spec)?;

                bridging_templates.insert(name.clone(), template);

                alias_spec
                    .properties
                    .iter()
                    .try_for_each(|prop| -> Result<(), anyhow::Error> {
                        match &*prop.type_annotation {
                            nullable_type @ TypeAnnotation::NullableTypeAnnotation {
                                type_annotation,
                            } => {
                                let key = nullable_type.as_cxx_type(&schema.module_name)?;

                                if nullable_bridging_templates.contains_key(&key) {
                                    return Ok(());
                                }

                                let bridging_template = cxx_nullable_bridging_template(
                                    &schema.module_name,
                                    &nullable_type.as_cxx_type(&schema.module_name)?,
                                    type_annotation,
                                )?;

                                nullable_bridging_templates.insert(key, bridging_template);

                                Ok(())
                            }
                            _ => Ok(()),
                        }
                    })?;
                Ok(())
            },
        )?;

        schema
            .enum_map
            .iter()
            .try_for_each(|(name, enum_spec)| -> Result<(), anyhow::Error> {
                enum_bridging_templates.insert(
                    enum_spec.name.clone(),
                    cxx_enum_bridging_template(&schema.module_name, name, enum_spec)?,
                );
                Ok(())
            })?;

        // C++ Templates are should be sorted in the order of their dependencies
        let mut ordered_templates = vec![];
        let ord = calc_deps_order(schema)?;
        println!("dependencies order: {:?}", ord);

        ordered_templates.extend(enum_bridging_templates.into_values());

        ord.iter().for_each(|name| {
            if let Some(template) = bridging_templates.remove(name) {
                ordered_templates.push(template);
            }

            if let Some(template) = nullable_bridging_templates.remove(&format!(
                "craby::{}::{}",
                flat_case(&schema.module_name),
                name
            )) {
                ordered_templates.push(template);
            }
        });

        ordered_templates.extend(bridging_templates.into_values());
        ordered_templates.extend(nullable_bridging_templates.into_values());

        Ok(ordered_templates)
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

        assert_snapshot!(result.func_extern_sigs.join("\n\n"));
        assert_snapshot!(result.func_impls.join("\n\n"));
    }

    #[test]
    fn test_get_rs_type_impls() {
        let schema = load_schema_json::<Schema>();
        let generator = CodeGenerator::new();
        let result = generator.get_rs_type_impls(&schema).unwrap();

        assert_snapshot!(result.into_values().collect::<Vec<_>>().join("\n"));
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
