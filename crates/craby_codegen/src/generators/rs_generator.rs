use std::collections::BTreeMap;

use craby_common::{
    constants::{crate_dir, impl_mod_name, HASH_COMMENT_PREFIX},
    utils::string::pascal_case,
};
use indoc::formatdoc;

use crate::{
    generators::types::TemplateResult,
    platform::rust::RsCxxBridge,
    types::{CodegenContext, CxxNamespace, Schema},
    utils::indent_str,
};

use super::types::{Generator, GeneratorInvoker, Template};

pub struct RsTemplate;
pub struct RsGenerator;

pub enum RsFileType {
    /// lib.rs
    CrateEntry,
    /// ffi.rs
    FFIEntry,
    /// generated.rs
    Generated,
    /// impl.rs
    ModImpl,
}

impl RsTemplate {
    fn impl_mods(&self, schemas: &[Schema]) -> Vec<String> {
        schemas
            .iter()
            .map(|schema| impl_mod_name(&schema.module_name))
            .collect::<Vec<String>>()
    }

    fn rs_cxx_bridges(&self, schemas: &[Schema]) -> Result<Vec<RsCxxBridge>, anyhow::Error> {
        let res = schemas
            .iter()
            .map(|schema| schema.as_rs_cxx_bridge())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(res)
    }

    /// Generates Rust FFI extern declarations for C++ bridging.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// #[cxx::bridge(namespace = "craby::mymodule::bridging")]
    /// pub mod bridging {
    ///     struct MyStruct {
    ///         foo: String,
    ///         bar: f64,
    ///     }
    ///
    ///     enum MyEnum {
    ///         Foo,
    ///         Bar,
    ///     }
    ///
    ///     extern "Rust" {
    ///         type MyModule;
    ///
    ///         #[cxx_name = "createMyModule"]
    ///         fn create_my_module(id: usize, data_path: &str) -> Box<MyModule>;
    ///
    ///         #[cxx_name = "multiply"]
    ///         fn my_module_multiply(it_: &mut MyModule, a: f64, b: f64) -> Result<f64>;
    ///     }
    /// }
    /// ```
    fn rs_cxx_extern(
        &self,
        cxx_ns: &CxxNamespace,
        rs_cxx_bridges: &[RsCxxBridge],
        has_signals: bool,
    ) -> String {
        let (impl_types, cxx_externs, struct_defs, enum_defs) = rs_cxx_bridges.iter().fold(
            (vec![], vec![], vec![], vec![]),
            |(mut impl_types, mut externs, mut structs, mut enums), bridge| {
                impl_types.push(bridge.impl_type.clone());
                externs.extend(bridge.func_extern_sigs.clone());
                structs.extend(bridge.struct_defs.clone());
                enums.extend(bridge.enum_defs.clone());
                (impl_types, externs, structs, enums)
            },
        );

        let cxx_extern_stmts = indent_str(&[impl_types, cxx_externs].concat().join("\n\n"), 4);
        let cxx_extern = formatdoc! {
            r#"
            extern "Rust" {{
            {cxx_extern_stmts}
            }}"#,
        };

        let cxx_signal_manager = if has_signals {
            formatdoc! {
                r#"
                #[namespace = "{cxx_ns}::signals"]
                unsafe extern "C++" {{
                    include!("CrabySignals.h");

                    type SignalManager;

                    fn emit(self: &SignalManager, id: usize, name: &str);
                    #[rust_name = "get_signal_manager"]
                    fn getSignalManager() -> &'static SignalManager;
                }}"#,
            }
        } else {
            String::new()
        };

        let code = indent_str(
            &[
                struct_defs.join("\n\n"),
                enum_defs.join("\n\n"),
                cxx_extern,
                cxx_signal_manager,
            ]
            .join("\n\n"),
            4,
        );

        formatdoc! {
            r#"
            #[cxx::bridge(namespace = "{cxx_ns}::bridging")]
            pub mod bridging {{
            {code}
            }}"#,
        }
    }

    /// Generates Rust FFI function implementations.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// fn create_my_module(id: usize, data_path: &str) -> Box<MyModule> {
    ///     let ctx = Context::new(id, data_path);
    ///     Box::new(MyModule::new(ctx))
    /// }
    ///
    /// fn my_module_multiply(it_: &mut MyModule, a: f64, b: f64) -> Result<f64> {
    ///     craby::catch_panic!({
    ///         let ret = it_.multiply(a, b);
    ///         ret
    ///     })
    /// }
    /// ```
    fn rs_cxx_impl(&self, rs_cxx_bridges: &[RsCxxBridge]) -> Vec<String> {
        rs_cxx_bridges
            .iter()
            .map(|bridge| bridge.func_impls.join("\n\n"))
            .collect::<Vec<_>>()
    }

    /// Generate the traits code for the given schema.
    ///
    /// ```rust,ignore
    /// pub trait MyModuleSpec {
    ///     fn multiply(&mut self, a: f64, b: f64) -> f64;
    /// }
    /// ```
    fn rs_spec(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let trait_name = pascal_case(&format!("{}Spec", schema.module_name));
        let mut methods = schema
            .methods
            .iter()
            .map(|spec| -> Result<String, anyhow::Error> {
                let sig = spec.try_into_impl_sig()?;
                Ok(format!("{sig};"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let signal_enum = if !schema.signals.is_empty() {
            let signal_enum_name = format!("{}Signal", schema.module_name);
            let (signal_members, pattern_matches): (Vec<String>, Vec<String>) = schema
                .signals
                .iter()
                .map(|signal| {
                    let member_name = pascal_case(&signal.name);
                    let enum_member = format!("{member_name},");
                    let enum_pattern_match = formatdoc! {
                        r#"{signal_enum_name}::{member_name} => manager.emit(self.id(), "{raw}"),"#,
                        raw = signal.name,
                    };

                    (enum_member, enum_pattern_match)
                })
                .unzip();

            let signal_members_exprs = indent_str(&signal_members.join("\n"), 4);
            let signal_enum = formatdoc! {
                r#"
                pub enum {signal_enum_name} {{
                {signal_members_exprs}
                }}"#,
            };

            let pattern_match_stmts = indent_str(&pattern_matches.join("\n"), 8);
            let emit_impl = formatdoc! {
                r#"
                fn emit(&self, signal_name: {signal_enum_name}) {{
                    let manager = crate::ffi::bridging::get_signal_manager();
                    match signal_name {{
                {pattern_match_stmts}
                    }}
                }}"#,
            };

            methods.insert(0, emit_impl);

            Some(signal_enum)
        } else {
            None
        };

        let method_defs = indent_str(&methods.join("\n"), 4);
        let spec_trait = formatdoc! {
            r#"
            pub trait {trait_name} {{
                fn new(ctx: Context) -> Self;
                fn id(&self) -> usize;
            {method_defs}
            }}"#
        };

        let content = [Some(spec_trait), signal_enum]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(content)
    }

    /// Generates default implementation structure for module.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// use craby::{prelude::*, throw};
    ///
    /// use crate::ffi::bridging::*;
    /// use crate::generated::*;
    ///
    /// pub struct MyModule {
    ///     ctx: Context,
    /// }
    ///
    /// impl MyModuleSpec for MyModule {
    ///     fn new(ctx: Context) -> Self {
    ///         MyModule { ctx }
    ///     }
    ///
    ///     fn id(&self) -> usize {
    ///         self.ctx.id
    ///     }
    ///
    ///     fn multiply(&mut self, a: Number, b: Number) -> Number {
    ///         unimplemented!();
    ///     }
    /// }
    /// ```
    fn rs_impl(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let struct_name = pascal_case(&schema.module_name);
        let trait_name = pascal_case(&format!("{}Spec", schema.module_name));
        let methods = schema
            .methods
            .iter()
            .map(|spec| -> Result<String, anyhow::Error> {
                let func_sig = spec.try_into_impl_sig()?;
                let code = formatdoc! {
                  r#"
                  {func_sig} {{
                      unimplemented!();
                  }}"#,
                };

                Ok(code)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let method_impls = indent_str(&methods.join("\n\n"), 4);
        let content = formatdoc! {
            r#"
            use craby::{{prelude::*, throw}};

            use crate::ffi::bridging::*;
            use crate::generated::*;

            pub struct {struct_name} {{
                ctx: Context,
            }}

            #[craby_module]
            impl {trait_name} for {struct_name} {{
            {method_impls}
            }}"#,
        };

        Ok(content)
    }

    /// Generate the `lib.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// pub(crate) mod generated;
    /// pub(crate) mod ffi;
    ///
    /// pub(crate) mod my_module_impl;
    /// ```
    fn lib_rs(&self, schemas: &[Schema]) -> Result<String, anyhow::Error> {
        let impl_mods = self
            .impl_mods(schemas)
            .iter()
            .map(|impl_mod| format!("pub(crate) mod {impl_mod};"))
            .collect::<Vec<String>>();

        let impl_mod_defs = impl_mods.join("\n");
        let content = formatdoc! {
            r#"
            #[rustfmt::skip]
            pub(crate) mod ffi;
            pub(crate) mod generated;

            {impl_mod_defs}"#,
        };

        Ok(content)
    }

    /// Generate the `ffi.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// use craby::prelude::*;
    ///
    /// use crate::my_module_impl::*;
    /// use crate::generated::*;
    ///
    /// use bridging::*;
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
    fn ffi_rs(&self, ctx: &CodegenContext) -> Result<String, anyhow::Error> {
        let cxx_ns = CxxNamespace::from(&ctx.project_name);
        let impl_mods = self
            .impl_mods(&ctx.schemas)
            .iter()
            .map(|impl_mod| format!("use crate::{impl_mod}::*;"))
            .collect::<Vec<String>>();

        let has_signals = ctx.schemas.iter().any(|schema| !schema.signals.is_empty());
        let rs_cxx_bridges = self.rs_cxx_bridges(&ctx.schemas)?;
        let cxx_impls = self.rs_cxx_impl(&rs_cxx_bridges);
        let cxx_externs = self.rs_cxx_extern(&cxx_ns, &rs_cxx_bridges, has_signals);
        let impl_mods = impl_mods.join("\n");
        let cxx_impls = cxx_impls.join("\n\n");
        let content = formatdoc! {
            r#"
            #[rustfmt::skip]
            use craby::prelude::*;

            {impl_mods}
            use crate::generated::*;

            use bridging::*;

            {cxx_externs}

            {cxx_impls}"#,
        };

        Ok(content)
    }

    /// Generate the `generated.rs` file for the given code generation results.
    ///
    /// ```rust,ignore
    /// use craby::prelude::*;
    ///
    /// use crate::ffi::bridging::*;
    ///
    /// pub trait MyModuleSpec {
    ///     fn multiply(&mut self, a: f64, b: f64) -> f64;
    /// }
    /// ```
    pub fn generated_rs(&self, schemas: &[Schema]) -> Result<String, anyhow::Error> {
        let mut spec_codes = Vec::with_capacity(schemas.len());
        let mut type_aliases = BTreeMap::new();

        for schema in schemas {
            // Collect the type implementations
            schema.try_collect_type_impls(&mut type_aliases)?;
            spec_codes.push(self.rs_spec(schema)?);
        }

        let hash = Schema::to_hash(schemas);
        let hash_comment = format!("{HASH_COMMENT_PREFIX} {hash}");
        let type_impls = type_aliases.into_values().collect::<Vec<_>>();

        let content = [
            vec![formatdoc! {
                r#"
                {hash_comment}
                #[rustfmt::skip]
                use craby::prelude::*;

                use crate::ffi::bridging::*;"#,
            }],
            spec_codes,
            type_impls,
        ]
        .concat()
        .join("\n\n");

        Ok(content)
    }
}

impl Template for RsTemplate {
    type FileType = RsFileType;

    fn render(
        &self,
        ctx: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<TemplateResult>, anyhow::Error> {
        let base_path = crate_dir(&ctx.root).join("src");
        let res = match file_type {
            RsFileType::CrateEntry => vec![TemplateResult {
                path: base_path.join("lib.rs"),
                content: self.lib_rs(&ctx.schemas)?,
                overwrite: false,
            }],
            RsFileType::FFIEntry => vec![TemplateResult {
                path: base_path.join("ffi.rs"),
                content: self.ffi_rs(ctx)?,
                overwrite: true,
            }],
            RsFileType::Generated => vec![TemplateResult {
                path: base_path.join("generated.rs"),
                content: self.generated_rs(&ctx.schemas)?,
                overwrite: true,
            }],
            RsFileType::ModImpl => ctx
                .schemas
                .iter()
                .map(|schema| -> Result<TemplateResult, anyhow::Error> {
                    let impl_code = self.rs_impl(schema)?;

                    Ok(TemplateResult {
                        path: base_path.join(format!("{}.rs", impl_mod_name(&schema.module_name))),
                        content: impl_code,
                        overwrite: false,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(res)
    }
}

impl Default for RsGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RsGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Generator<RsTemplate> for RsGenerator {
    fn cleanup(_: &CodegenContext) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn generate(&self, ctx: &CodegenContext) -> Result<Vec<TemplateResult>, anyhow::Error> {
        let template = self.template_ref();
        let res = [
            template.render(ctx, &RsFileType::CrateEntry)?,
            template.render(ctx, &RsFileType::FFIEntry)?,
            template.render(ctx, &RsFileType::Generated)?,
            template.render(ctx, &RsFileType::ModImpl)?,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        Ok(res)
    }

    fn template_ref(&self) -> &RsTemplate {
        &RsTemplate
    }
}

impl GeneratorInvoker for RsGenerator {
    fn invoke_generate(&self, ctx: &CodegenContext) -> Result<Vec<TemplateResult>, anyhow::Error> {
        self.generate(ctx)
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
