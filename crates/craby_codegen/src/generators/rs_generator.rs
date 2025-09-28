use std::{collections::BTreeMap, path::PathBuf};

use craby_common::{
    constants::{crate_dir, impl_mod_name},
    utils::string::pascal_case,
};
use indoc::formatdoc;

use crate::{
    platform::rust::RsCxxBridge,
    types::{CodegenContext, Schema},
    utils::indent_str,
};

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

    fn rs_cxx_extern(&self, rs_cxx_bridges: &Vec<RsCxxBridge>, has_signals: bool) -> String {
        let mut cxx_extern = vec![];
        let mut struct_defs = vec![];
        let mut enum_defs = vec![];

        rs_cxx_bridges.iter().for_each(|bridge| {
            cxx_extern.extend(bridge.func_extern_sigs.clone());
            struct_defs.extend(bridge.struct_defs.clone());
            enum_defs.extend(bridge.enum_defs.clone());
        });

        let cxx_extern = formatdoc! {
            r#"
            extern "Rust" {{
            {cxx_extern}
            }}
            "#,
            cxx_extern = indent_str(cxx_extern.join("\n\n"), 4),
        };

        let cxx_signal_manager = if has_signals {
            formatdoc! {
                r#"
                #[namespace = "craby::signals"]
                unsafe extern "C++" {{
                    include!("signals.h");

                    type SignalManager;

                    fn emit(self: &SignalManager, id: usize, name: &str);
                    #[rust_name = "get_signal_manager"]
                    fn getSignalManager() -> &'static SignalManager;
                }}"#,
            }
        } else {
            String::new()
        };

        let code = [
          struct_defs.join("\n\n"),
          enum_defs.join("\n\n"),
          cxx_extern,
          cxx_signal_manager,
        ].join("\n\n");

        formatdoc! {
            r#"
            #[cxx::bridge(namespace = "craby::bridging")]
            pub mod bridging {{
            {code}
            }}"#,
            code = indent_str(code, 4),
        }
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
    ///     fn multiply(&self, a: f64, b: f64) -> f64;
    /// }
    /// ```
    fn rs_spec(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());
        let methods = schema
            .methods
            .iter()
            .map(|spec| -> Result<String, anyhow::Error> {
                let sig = spec.as_impl_sig()?;
                Ok(format!("{};", sig))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let (signal_enum, emit_impl) = if schema.signals.len() > 0 {
            let signal_enum_name = format!("{}Signal", schema.module_name);
            let (signal_members, pattern_matches): (Vec<String>, Vec<String>) = schema
                .signals
                .iter()
                .map(|signal| {
                    let member_name = pascal_case(&signal.name);
                    let enum_member = format!("{},", member_name);
                    let enum_pattern_match = formatdoc! {
                        r#"{signal_enum_name}::{member_name} => manager.emit(self.id(), "{raw}"),"#,
                        signal_enum_name = signal_enum_name,
                        member_name = member_name,
                        raw = signal.name,
                    };

                    (enum_member, enum_pattern_match)
                })
                .unzip();

            let signal_enum = formatdoc! {
                r#"
                pub enum {signal_enum_name} {{
                    {signal_members}
                }}
                "#,
                signal_enum_name = signal_enum_name,
                signal_members = signal_members.join("\n"),
            };

            let emit_impl = formatdoc! {
                r#"
                fn emit(&self, signal_name: {signal_enum_name}) {{
                    let manager = crate::ffi::bridging::get_signal_manager();
                    match signal_name {{
                {pattern_matches}
                    }}
                }}"#,
                signal_enum_name = signal_enum_name,
                pattern_matches = indent_str(pattern_matches.join("\n"), 8),
            };

            (signal_enum, emit_impl)
        } else {
            (String::new(), String::new())
        };

        let content = formatdoc! {
            r#"
            {signal_enum}
            pub trait {trait_name} {{
                fn new(id: usize) -> Self;
                fn id(&self) -> usize;
            {emit_impl}
            {methods}
            }}"#,
            trait_name = trait_name,
            signal_enum = signal_enum,
            emit_impl = indent_str(emit_impl, 4),
            methods = indent_str(methods.join("\n"), 4),
        };

        Ok(content)
    }

    fn rs_impl(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let mod_name = pascal_case(schema.module_name.as_str());
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());

        let methods = schema
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
        // use crate::ffi::bridging::*;
        // use crate::generated::*;
        // use crate::types::*;
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
            use crate::ffi::bridging::*;
            use crate::generated::*;
            use crate::types::*;

            pub struct {mod_name} {{
                id: usize,
            }};

            impl {trait_name} for {mod_name} {{
                fn new(id: usize) -> Self {{
                    {mod_name} {{ id }}
                }}

                fn id(&self) -> usize {{
                    self.id
                }}

            {methods}
            }}"#,
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

        let has_signals = schemas.iter().any(|schema| schema.signals.len() > 0);
        let rs_cxx_bridges = self.rs_cxx_bridges(schemas)?;
        let cxx_externs = self.rs_cxx_extern(&rs_cxx_bridges, has_signals);
        let cxx_impls = self.rs_cxx_impl(&rs_cxx_bridges);

        let content = formatdoc! {
            r#"
            #[rustfmt::skip]
            {impl_mods}
            use crate::generated::*;

            use bridging::*;

            {cxx_externs}

            {cxx_impl}"#,
            impl_mods = impl_mods.join("\n"),
            cxx_externs = cxx_externs,
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
        let mut type_aliases = BTreeMap::new();

        schemas
            .iter()
            .try_for_each(|schema| -> Result<(), anyhow::Error> {
                let spec = self.rs_spec(schema)?;

                // Collect the type implementations
                schema.as_rs_type_impls(&mut type_aliases)?;

                spec_codes.push(spec);

                Ok(())
            })?;

        let type_impls = type_aliases.into_values().collect::<Vec<_>>();
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
        project: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error> {
        let path = self.file_path(file_type);
        let content = match file_type {
            RsFileType::CrateEntry => self.lib_rs(&project.schemas),
            RsFileType::FFIEntry => self.ffi_rs(&project.schemas),
            RsFileType::Generated => self.generated_rs(&project.schemas),
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
    fn generate(&self, project: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error> {
        let base_path = crate_dir(&project.root).join("src");
        let template = self.template_ref();
        let mut res = [
            template.render(project, &RsFileType::CrateEntry)?,
            template.render(project, &RsFileType::FFIEntry)?,
            template.render(project, &RsFileType::Generated)?,
            template.render(project, &RsFileType::Types)?,
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
            project
                .schemas
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
        project: &CodegenContext,
    ) -> Result<Vec<GenerateResult>, anyhow::Error> {
        self.generate(project)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::tests::get_codegen_context;

    use super::*;

    #[test]
    fn test_rs_generator() {
        let ctx = get_codegen_context();
        let generator = RsGenerator::new();
        let results = generator.generate(&ctx).unwrap();
        let result = results
            .iter()
            .map(|res| format!("{}\n{}", res.path.display(), res.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        assert_snapshot!(result);
    }
}
