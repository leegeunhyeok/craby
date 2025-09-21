use std::path::PathBuf;

use craby_common::{
    constants::{crate_dir, impl_mod_name},
    utils::string::{pascal_case, snake_case},
};
use indoc::formatdoc;

use crate::{platform::rust::RsCxxBridge, types::schema::Schema, utils::indent_str};

use super::types::{GenerateResult, Generator, GeneratorInvoker, Template};

pub struct RsTemplate;
pub struct RsGenerator;

pub enum RsFileType {
    /// lib.rs
    CrateEntry,
    /// ffi.rs
    FFIEntry,
    /// types.rs
    Types,
    /// generated.rs
    Generated,
}

impl RsTemplate {
    fn file_path(&self, file_type: &RsFileType) -> PathBuf {
        match file_type {
            RsFileType::CrateEntry => PathBuf::from("lib.rs"),
            RsFileType::FFIEntry => PathBuf::from("ffi.rs"),
            RsFileType::Generated => PathBuf::from("generated.rs"),
            RsFileType::Types => PathBuf::from("types.rs"),
        }
    }

    fn impl_mods(&self, schemas: &Vec<Schema>) -> Vec<String> {
        schemas
            .iter()
            .map(|schema| impl_mod_name(&schema.module_name))
            .collect::<Vec<String>>()
    }

    fn rs_cxx_bridges(&self, schemas: &Vec<Schema>) -> Result<Vec<RsCxxBridge>, anyhow::Error> {
        let res = schemas
            .iter()
            .map(|schema| schema.as_rs_cxx_bridge())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(res)
    }

    fn rs_cxx_extern(&self, rs_cxx_bridges: &Vec<RsCxxBridge>) -> Vec<String> {
        rs_cxx_bridges
            .iter()
            .map(|bridge| {
                let cxx_extern = bridge.func_extern_sigs.join("\n\n");
                let struct_defs = bridge.struct_defs.join("\n\n");
                let enum_defs = bridge.enum_defs.join("\n\n");

                formatdoc! {
                    r#"
                    #[cxx::bridge(namespace = "craby::bridging")]
                    pub mod bridging {{
                        // Type definitions
                    {struct_defs}

                    {enum_defs}

                        extern "Rust" {{
                    {cxx_extern}
                        }}
                    }}"#,
                    struct_defs = indent_str(struct_defs, 4),
                    enum_defs = indent_str(enum_defs, 4),
                    cxx_extern = indent_str(cxx_extern, 8),
                }
            })
            .collect::<Vec<_>>()
    }

    fn rs_cxx_impl(&self, rs_cxx_bridges: &Vec<RsCxxBridge>) -> Vec<String> {
        rs_cxx_bridges
            .iter()
            .map(|bridge| bridge.func_impls.join("\n\n"))
            .collect::<Vec<_>>()
    }

    /// Generate the traits code for the given schema.
    ///
    /// ```rust,ignore
    /// pub trait MyModuleSpec {
    ///     fn multiply(a: f64, b: f64) -> f64;
    /// }
    /// ```
    fn rs_spec(&self, schema: &Schema) -> Result<String, anyhow::Error> {
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

        let content = formatdoc! {
            r#"
            pub trait {trait_name} {{
            {methods}
            }}"#,
            trait_name = trait_name,
            methods = indent_str(methods.join("\n"), 4),
        };

        Ok(content)
    }

    fn rs_impl(&self, schema: &Schema) -> Result<String, anyhow::Error> {
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
        let content = formatdoc! {
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

        Ok(content)
    }

    /// Generate the `lib.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// pub(crate) mod generated;
    /// pub(crate) mod ffi;
    /// pub(crate) mod my_module_impl;
    /// ```
    fn lib_rs(&self, schemas: &Vec<Schema>) -> Result<String, anyhow::Error> {
        let impl_mods = self
            .impl_mods(schemas)
            .iter()
            .map(|impl_mod| format!("pub(crate) mod {};", impl_mod))
            .collect::<Vec<String>>();

        let content = formatdoc! {
            r#"
            #[rustfmt::skip]
            pub(crate) mod ffi;
            pub(crate) mod generated;
            pub(crate) mod types;

            {impl_mods}"#,
            impl_mods = impl_mods.join("\n"),
        };

        Ok(content)
    }

    /// Generate the `ffi.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// use ffi::*;
    /// use crate::generated::*;
    /// use crate::my_module_impl::*;
    ///
    /// #[cxx::bridge(namespace = "craby::mymodule")]
    /// pub mod bridging {
    ///     extern "Rust" {
    ///         #[cxx_name = "numericMethod"]
    ///         fn my_module_numeric_method(arg: f64) -> f64;
    ///     }
    /// }
    ///
    /// fn my_module_numeric_method(arg: f64) -> f64 {
    ///     MyModule::numeric_method(arg)
    /// }
    /// ```
    fn ffi_rs(&self, schemas: &Vec<Schema>) -> Result<String, anyhow::Error> {
        let impl_mods = self
            .impl_mods(schemas)
            .iter()
            .map(|impl_mod| format!("use crate::{}::*;", impl_mod))
            .collect::<Vec<String>>();

        let rs_cxx_bridges = self.rs_cxx_bridges(schemas)?;
        let cxx_externs = self.rs_cxx_extern(&rs_cxx_bridges);
        let cxx_impls = self.rs_cxx_impl(&rs_cxx_bridges);

        let content = formatdoc! {
            r#"
            #[rustfmt::skip]
            {impl_mods}
            use crate::generated::*;

            use bridging::*;

            {cxx_extern}

            {cxx_impl}"#,
            impl_mods = impl_mods.join("\n"),
            cxx_extern = cxx_externs.join("\n\n"),
            cxx_impl = cxx_impls.join("\n\n"),
        };

        Ok(content)
    }

    /// Generate the `types.rs`
    fn types_rs(&self) -> String {
        formatdoc! {
            r#"
            #[rustfmt::skip]
            pub type Boolean = bool;
            pub type Number = f64;
            pub type String = std::string::String;
            pub type Array<T> = Vec<T>;
            pub type Promise<T> = Result<T, anyhow::Error>;
            pub type Void = ();

            pub mod promise {{
                use super::Promise;

                pub fn resolve<T>(val: T) -> Promise<T> {{
                    Ok(val)
                }}

                pub fn rejected<T>(err: impl AsRef<str>) -> Promise<T> {{
                    Err(anyhow::anyhow!(err.as_ref().to_string()))
                }}
            }}

            pub struct Nullable<T> {{
                val: Option<T>,
            }}

            impl<T> Nullable<T> {{
                pub fn new(val: Option<T>) -> Self {{
                    Nullable {{ val }}
                }}

                pub fn some(val: T) -> Self {{
                    Nullable {{ val: Some(val) }}
                }}

                pub fn none() -> Self {{
                    Nullable {{ val: None }}
                }}

                pub fn value(mut self, val: T) -> Self {{
                    self.val = Some(val);
                    self
                }}

                pub fn value_of(&self) -> Option<&T> {{
                    self.val.as_ref()
                }}

                pub fn into_value(self) -> Option<T> {{
                    self.val
                }}
            }}"#
        }
    }

    /// Generate the `generated.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// use crate::ffi::bridging::*;
    /// use crate::types::*;
    ///
    /// pub trait MyModuleSpec {
    ///     fn multiply(a: f64, b: f64) -> f64;
    /// }
    /// ```
    pub fn generated_rs(&self, schemas: &Vec<Schema>) -> Result<String, anyhow::Error> {
        let mut spec_codes = vec![];
        let mut type_impls = vec![];

        schemas
            .iter()
            .try_for_each(|schema| -> Result<(), anyhow::Error> {
                let spec = self.rs_spec(schema)?;
                let impls = schema.as_rs_type_impls()?.into_values();

                spec_codes.push(spec);
                type_impls.extend(impls);

                Ok(())
            })?;

        let content = formatdoc! {
            r#"
            #[rustfmt::skip]
            use crate::ffi::bridging::*;
            use crate::types::*;

            {spec_codes}

            {type_impls}"#,
            type_impls = type_impls.join("\n\n"),
            spec_codes = spec_codes.join("\n\n"),
        };

        Ok(content)
    }
}

impl Template for RsTemplate {
    type FileType = RsFileType;

    fn render(
        &self,
        schemas: &Vec<Schema>,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let path = self.file_path(file_type);
        let content = match file_type {
            RsFileType::CrateEntry => self.lib_rs(schemas),
            RsFileType::FFIEntry => self.ffi_rs(schemas),
            RsFileType::Generated => self.generated_rs(schemas),
            RsFileType::Types => Ok(self.types_rs()),
        }?;

        Ok(vec![(path, content)])
    }
}

impl RsGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator<RsTemplate> for RsGenerator {
    fn generate(
        &self,
        project_root: &PathBuf,
        schemas: &Vec<Schema>,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let base_path = crate_dir(project_root).join("src");
        let template = self.template_ref();
        let mut res = [
            template.render(schemas, &RsFileType::CrateEntry)?,
            template.render(schemas, &RsFileType::FFIEntry)?,
            template.render(schemas, &RsFileType::Generated)?,
            template.render(schemas, &RsFileType::Types)?,
        ]
        .into_iter()
        .flatten()
        .map(|(path, content)| GenerateResult {
            path: base_path.join(path),
            content,
            overwrite: true,
        })
        .collect::<Vec<_>>();

        res.extend(
            schemas
                .iter()
                .map(|schema| -> Result<GenerateResult, anyhow::Error> {
                    let impl_code = template.rs_impl(schema)?;

                    Ok(GenerateResult {
                        path: base_path.join(format!("{}.rs", impl_mod_name(&schema.module_name))),
                        content: impl_code,
                        overwrite: false,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );

        Ok(res)
    }

    fn template_ref(&self) -> &RsTemplate {
        &RsTemplate
    }
}

impl GeneratorInvoker for RsGenerator {
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
    fn test_rs_generator() {
        let schema = load_schema_json::<Schema>();
        let generator = RsGenerator::new();
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
