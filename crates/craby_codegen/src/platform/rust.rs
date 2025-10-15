use std::collections::BTreeMap;

use craby_common::utils::string::{camel_case, pascal_case, snake_case};
use indoc::formatdoc;
use rustc_hash::FxHashMap;

use crate::{
    constants::specs::RESERVED_ARG_NAME_MODULE,
    parser::types::{
        EnumTypeAnnotation, Method, ObjectTypeAnnotation, Param, RefTypeAnnotation, TypeAnnotation,
    },
    platform::rust::template::{alias_default_impl, as_struct_def, enum_default_impl},
    types::Schema,
    utils::indent_str,
};

#[derive(Debug)]
pub struct RsType(pub String);

#[derive(Debug)]
pub struct RsBridgeType(pub String);

#[derive(Debug)]
pub struct RsImplType(pub String);

/// Collection of Rust code for FFI.
#[derive(Debug, Clone)]
pub struct RsCxxBridge {
    /// The impl struct type name.
    ///
    /// ```rust,ignore
    /// type MyModule;
    /// ```
    pub impl_type: String,
    /// The struct definition.
    ///
    /// ```rust,ignore
    /// struct MyStruct {
    ///   foo: String,
    ///   bar: f64,
    ///   baz: bool,
    /// }
    /// ```
    pub struct_defs: Vec<String>,
    /// The enum definition.
    ///
    /// ```rust,ignore
    /// enum MyEnum {
    ///   Foo,
    ///   Bar,
    ///   Baz,
    /// }
    /// ```
    pub enum_defs: Vec<String>,
    /// The extern function declaration.
    ///
    /// **Example**
    ///
    /// ```rust,ignore
    /// #[cxx_name = "myFunc"]
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Result<Baz>;
    /// ```
    pub func_extern_sigs: Vec<String>,
    /// The implementation function of the extern function.
    ///
    /// **Example**
    ///
    /// ```rust,ignore
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Result<Baz> {
    ///   MyModule::my_func(arg1, arg2)
    /// }
    /// ```
    pub func_impls: Vec<String>,
}

impl TypeAnnotation {
    /// Converts TypeAnnotation to Rust type representation.
    ///
    /// # Generated Code Examples
    ///
    /// ```rust,ignore
    /// bool                          // Boolean
    /// f64                           // Number
    /// String                        // String
    /// Vec<f64>                      // Array<Number>
    /// MyEnum                        // Enum
    /// MyStruct                      // Object
    /// NullableNumber                // Nullable<Number>
    /// Result<f64, anyhow::Error>    // Promise<Number>
    /// ```
    pub fn as_rs_type(&self) -> Result<RsType, anyhow::Error> {
        let rs_type = match self {
            TypeAnnotation::Void => "()".to_string(),
            TypeAnnotation::Boolean => "bool".to_string(),
            TypeAnnotation::Number => "f64".to_string(),
            TypeAnnotation::String => "String".to_string(),
            TypeAnnotation::Array(element_type) => {
                if let TypeAnnotation::Array(..) = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Vec<{}>", element_type.as_rs_type()?.0)
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Promise(resolve_type) => {
                format!("Result<{}, anyhow::Error>", resolve_type.as_rs_type()?.0)
            }
            TypeAnnotation::Nullable(type_annotation) => match &**type_annotation {
                TypeAnnotation::Boolean => "NullableBoolean".to_string(),
                TypeAnnotation::Number => "NullableNumber".to_string(),
                TypeAnnotation::String => "NullableString".to_string(),
                TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                    format!("Nullable{}", name)
                }
                TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                    format!("Nullable{}", name)
                }
                TypeAnnotation::Ref(RefTypeAnnotation { name, .. }) => {
                    format!("Nullable{}", name)
                }
                TypeAnnotation::Array(element_type) => match &**element_type {
                    TypeAnnotation::Boolean => "NullableBooleanArray".to_string(),
                    TypeAnnotation::Number => "NullableNumberArray".to_string(),
                    TypeAnnotation::String => "NullableStringArray".to_string(),
                    TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                        format!("Nullable{}Array", name)
                    }
                    TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                        format!("Nullable{}Array", name)
                    }
                    TypeAnnotation::Ref(RefTypeAnnotation { name, .. }) => {
                        format!("Nullable{}Array", name)
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                        "[as_rs_type] Unsupported type annotation for nullable array type: {:?}",
                        element_type
                    ))
                    }
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "[as_rs_type] Unsupported type annotation for nullable type: {:?}",
                        type_annotation
                    ))
                }
            },
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_rs_type] Unsupported type annotation: {:?}",
                    self
                ));
            }
        };

        Ok(RsType(rs_type))
    }

    /// Converts TypeAnnotation to Rust FFI bridge type for cxx extern.
    ///
    /// # Generated Code Examples
    ///
    /// ```rust,ignore
    /// bool                          // Boolean
    /// f64                           // Number
    /// String                        // String
    /// Result<f64>                   // Promise<Number> (shortened for FFI)
    /// ```
    pub fn as_rs_bridge_type(&self) -> Result<RsBridgeType, anyhow::Error> {
        let extern_type = match self {
            TypeAnnotation::Promise(resolve_type) => {
                format!("Result<{}>", resolve_type.as_rs_type()?.0)
            }
            _ => self.as_rs_type()?.0,
        };

        Ok(RsBridgeType(extern_type))
    }

    /// Converts TypeAnnotation to user-facing Rust implementation type.
    ///
    /// # Generated Code Examples
    ///
    /// ```rust,ignore
    /// Boolean                       // Boolean (aliased bool)
    /// Number                        // Number (aliased f64)
    /// String                        // String
    /// Array<Number>                 // Array<Number>
    /// Promise<Number>               // Promise<Number>
    /// Nullable<Number>              // Nullable<Number>
    /// ```
    pub fn as_rs_impl_type(&self) -> Result<RsImplType, anyhow::Error> {
        let rs_type = match self {
            TypeAnnotation::Void => "Void".to_string(),
            TypeAnnotation::Boolean => "Boolean".to_string(),
            TypeAnnotation::Number => "Number".to_string(),
            TypeAnnotation::String => "String".to_string(),
            TypeAnnotation::Array(element_type) => {
                if let TypeAnnotation::Array { .. } = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Array<{}>", element_type.as_rs_impl_type()?.0)
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Promise(resolved_type) => {
                format!("Promise<{}>", resolved_type.as_rs_impl_type()?.0)
            }
            TypeAnnotation::Nullable(type_annotation) => {
                let type_annotation = type_annotation.as_rs_impl_type()?.0;
                format!("Nullable<{}>", type_annotation)
            }
            TypeAnnotation::Ref(..) => unreachable!(),
        };
        Ok(RsImplType(rs_type))
    }

    /// Generates default value for Rust types.
    ///
    /// # Generated Code Examples
    ///
    /// ```rust,ignore
    /// false                         // Boolean
    /// 0.0                           // Number
    /// String::default()             // String
    /// Vec::default()                // Array
    /// MyEnum::default()             // Enum
    /// MyStruct::default()           // Object
    /// NullableNumber::default()     // Nullable<Number>
    /// ```
    pub fn as_rs_default_val(&self) -> Result<String, anyhow::Error> {
        let default_val = match self {
            TypeAnnotation::Boolean => "false".to_string(),
            TypeAnnotation::Number => "0.0".to_string(),
            TypeAnnotation::String => "String::default()".to_string(),
            TypeAnnotation::Array(..) => "Vec::default()".to_string(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                format!("{}::default()", name)
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                format!("{}::default()", name)
            }
            TypeAnnotation::Nullable(..) => {
                let nullable_type = self.as_rs_type()?.0;
                format!("{}::default()", nullable_type)
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "[as_rs_default_val] Unsupported type annotation: {:?}",
                    self
                ))
            }
        };

        Ok(default_val)
    }
}

impl Method {
    /// Converts Method to Rust trait method signature.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// fn multiply(&mut self, a: Number, b: Number) -> Number
    /// fn add_async(&mut self, a: Number, b: Number) -> Promise<Number>
    /// ```
    pub fn try_into_impl_sig(&self) -> Result<String, anyhow::Error> {
        let return_type = self.ret_type.as_rs_impl_type()?.0;
        let params_sig = std::iter::once("&mut self".to_string())
            .chain(
                self.params
                    .iter()
                    .map(|param| param.try_into_impl_sig())
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .collect::<Vec<_>>()
            .join(", ");

        let fn_name = snake_case(&self.name);
        let ret_annotation = if return_type == "()" {
            String::new()
        } else {
            format!(" -> {}", return_type)
        };

        Ok(format!(
            "fn {}({}){}",
            fn_name,
            params_sig,
            ret_annotation
        ))
    }
}

impl Param {
    /// Converts parameter to FFI function signature.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// a: f64
    /// name: String
    /// items: Vec<MyStruct>
    /// ```
    pub fn try_into_cxx_sig(&self) -> Result<String, anyhow::Error> {
        let param_type = self.type_annotation.as_rs_type()?.0;
        Ok(format!("{}: {}", snake_case(&self.name), param_type))
    }

    /// Converts parameter to implementation function signature.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// a: Number
    /// name: String
    /// items: Array<MyStruct>
    /// ```
    pub fn try_into_impl_sig(&self) -> Result<String, anyhow::Error> {
        let param_type = self.type_annotation.as_rs_impl_type()?.0;
        Ok(format!("{}: {}", snake_case(&self.name), param_type))
    }
}

impl Schema {
    /// Generates complete Rust FFI bridge including externs, structs, enums, and implementations.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// // In ffi.rs extern block:
    /// type MyModule;
    ///
    /// #[cxx_name = "createMyModule"]
    /// fn create_my_module(id: usize) -> Box<MyModule>;
    ///
    /// #[cxx_name = "multiply"]
    /// fn my_module_multiply(it_: &mut MyModule, a: f64, b: f64) -> Result<f64>;
    ///
    /// // Implementation:
    /// fn create_my_module(id: usize) -> Box<MyModule> {
    ///     Box::new(MyModule::new(id))
    /// }
    ///
    /// fn my_module_multiply(it_: &mut MyModule, a: f64, b: f64) -> Result<f64> {
    ///     catch_panic!({
    ///         let ret = it_.multiply(a, b);
    ///         ret
    ///     })
    /// }
    /// ```
    pub fn as_rs_cxx_bridge(&self) -> Result<RsCxxBridge, anyhow::Error> {
        let mut func_extern_sigs = Vec::with_capacity(self.methods.len() + 1);
        let mut func_impls = Vec::with_capacity(self.methods.len() + 1);
        let mut type_impls = vec![];
        let mut struct_defs = FxHashMap::default();

        func_extern_sigs.push(formatdoc! {
            r#"
            #[cxx_name = "create{module_name}"]
            fn create_{snake_cake}(id: usize) -> Box<{module_name}>;"#,
            module_name = pascal_case(&self.module_name),
            snake_cake = snake_case(&self.module_name),
        });

        func_impls.push(formatdoc! {
            r#"
            fn create_{snake_cake}(id: usize) -> Box<{module_name}> {{
                Box::new({module_name}::new(id))
            }}"#,
            module_name = pascal_case(&self.module_name),
            snake_cake = snake_case(&self.module_name),
        });

        // Collect extern function signatures and implementations
        for method_spec in &self.methods {
            // Collect nullable parameters
            for param in &method_spec.params {
                if let nullable_type @ TypeAnnotation::Nullable(type_annotation) =
                    &param.type_annotation
                {
                    if struct_defs.contains_key(nullable_type) {
                        continue;
                    }

                    let struct_type = nullable_type.as_rs_bridge_type()?.0;
                    let base_type = type_annotation.as_rs_type()?.0;
                    let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                    let default_val = type_annotation.as_rs_default_val()?;

                    struct_defs.insert(
                        nullable_type.clone(),
                        formatdoc! {
                            r#"
                            struct {struct_type} {{
                                null: bool,
                                val: {base_type},
                            }}"#,
                            struct_type = struct_type,
                            base_type = base_type,
                        },
                    );

                    let nullable_impl = formatdoc! {
                        r#"
                        impl From<{struct_type}> for Nullable<{rs_impl_type}> {{
                            fn from(val: {struct_type}) -> Self {{
                                Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                            }}
                        }}

                        impl From<Nullable<{rs_impl_type}>> for {struct_type} {{
                            fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                                let val = val.into_value();
                                let null = val.is_none();
                                {struct_type} {{
                                    val: val.unwrap_or({default_val}),
                                    null,
                                }}
                            }}
                        }}"#,
                        struct_type = struct_type,
                        rs_impl_type = rs_impl_type,
                        default_val = default_val,
                    };

                    type_impls.push(nullable_impl);
                }
            }

            let ret_type = method_spec.ret_type.as_rs_type()?.0;
            let ret_type = match method_spec.ret_type {
                TypeAnnotation::Promise(_) => ret_type,
                _ => format!("Result<{}, anyhow::Error>", ret_type),
            };
            let ret_extern_type = method_spec.ret_type.as_rs_bridge_type()?.0;
            let ret_extern_type = match method_spec.ret_type {
                TypeAnnotation::Promise(_) => ret_extern_type,
                _ => format!("Result<{}>", ret_extern_type),
            };

            let params_sig = method_spec
                .params
                .iter()
                .map(|param| param.try_into_cxx_sig())
                .collect::<Result<Vec<_>, _>>()
                .map(|mut params| {
                    params.insert(
                        0,
                        format!(
                            "{}: &mut {}",
                            RESERVED_ARG_NAME_MODULE,
                            pascal_case(&self.module_name)
                        ),
                    );
                    params.join(", ")
                })?;

            let mod_name = snake_case(&self.module_name);
            let fn_name = snake_case(&method_spec.name);
            let fn_args = method_spec
                .params
                .iter()
                .map(|param| {
                    let name = snake_case(&param.name);
                    if let TypeAnnotation::Nullable(..) = &param.type_annotation {
                        format!("{}.into()", name)
                    } else {
                        name
                    }
                })
                .collect::<Vec<_>>();
            let prefixed_fn_name = format!("{}_{}", mod_name, fn_name);
            let ret_extern_annotation = format!(" -> {}", ret_extern_type);
            let ret_annotation = format!(" -> {}", ret_type);
            let extern_func = formatdoc! {
                r#"
                #[cxx_name = "{cxx_extern_fn_name}"]
                fn {prefixed_fn_name}({params_sig}){ret};"#,
                cxx_extern_fn_name = camel_case(&method_spec.name),
                prefixed_fn_name = prefixed_fn_name,
                params_sig = params_sig,
                ret = ret_extern_annotation,
            };

            let ret = if let TypeAnnotation::Nullable(..) = &method_spec.ret_type {
                "ret.into()"
            } else {
                "ret"
            };

            let impl_func = match method_spec.ret_type {
                TypeAnnotation::Promise(_) => formatdoc! {
                    r#"
                    fn {prefixed_fn_name}({params_sig}){ret_type} {{
                        catch_panic!({{
                            let ret = {it}.{fn_name}({fn_args});
                            {ret}
                        }}).and_then(|r| r)
                    }}"#,
                    params_sig = params_sig,
                    it = RESERVED_ARG_NAME_MODULE,
                    ret_type = ret_annotation,
                    ret = ret,
                    prefixed_fn_name = prefixed_fn_name,
                    fn_name = fn_name.to_string(),
                    fn_args = fn_args.join(", "),
                },
                _ => formatdoc! {
                    r#"
                    fn {prefixed_fn_name}({params_sig}){ret_type} {{
                        catch_panic!({{
                            let ret = {it}.{fn_name}({fn_args});
                            {ret}
                        }})
                    }}"#,
                    params_sig = params_sig,
                    it = RESERVED_ARG_NAME_MODULE,
                    ret_type = ret_annotation,
                    ret = ret,
                    prefixed_fn_name = prefixed_fn_name,
                    fn_name = fn_name.to_string(),
                    fn_args = fn_args.join(", "),
                },
            };

            func_extern_sigs.push(extern_func);
            func_impls.push(impl_func);
        }

        // Collect alias types (struct)
        for type_annotation in &self.aliases {
            if !struct_defs.contains_key(type_annotation) {
                let obj = type_annotation.as_object().unwrap();
                struct_defs.insert(type_annotation.clone(), as_struct_def(obj)?);
                type_impls.push(alias_default_impl(obj)?);
            }
        }

        // Collect enum types
        let enum_defs = self
            .enums
            .iter()
            .map(|type_annotation| {
                let enum_schema = type_annotation.as_enum().unwrap();
                let members = enum_schema
                    .members
                    .iter()
                    .map(|m| format!("{},", m.name))
                    .collect::<Vec<_>>()
                    .join("\n");

                formatdoc! {
                    r#"
                    enum {name} {{
                    {members}
                    }}"#,
                    name = enum_schema.name,
                    members = indent_str(members, 4),
                }
            })
            .collect();

        Ok(RsCxxBridge {
            impl_type: format!("type {};", pascal_case(&self.module_name)),
            struct_defs: struct_defs.into_values().collect(),
            enum_defs,
            func_extern_sigs,
            func_impls,
        })
    }

    /// Collects and generates all type implementations (Default, From traits).
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// impl Default for NullableNumber {
    ///     fn default() -> Self {
    ///         NullableNumber {
    ///             null: true,
    ///             val: 0.0,
    ///         }
    ///     }
    /// }
    ///
    /// impl From<NullableNumber> for Nullable<Number> {
    ///     fn from(val: NullableNumber) -> Self {
    ///         Nullable::new(if val.null { None } else { Some(val.val) })
    ///     }
    /// }
    /// ```
    pub fn try_collect_type_impls(
        &self,
        type_impls: &mut BTreeMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        // Collect extern function signatures and implementations
        for method_spec in &self.methods {
            for param in &method_spec.params {
                // Collect nullable parameters
                if let nullable_type @ TypeAnnotation::Nullable(type_annotation) =
                    &param.type_annotation
                {
                    let rs_type = type_annotation.as_rs_type()?.0;

                    if let std::collections::btree_map::Entry::Vacant(e) = type_impls.entry(rs_type) {
                        let nullable_type = nullable_type.as_rs_bridge_type()?.0;
                        let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                        let default_val = type_annotation.as_rs_default_val()?;

                        let default_impl = formatdoc! {
                            r#"
                            impl Default for {nullable_type} {{
                                fn default() -> Self {{
                                    {nullable_type} {{
                                        null: true,
                                        val: {default_val},
                                    }}
                                }}
                            }}"#,
                            nullable_type = nullable_type,
                            default_val = default_val,
                        };

                        let nullable_impl = formatdoc! {
                            r#"
                            impl From<{nullable_type}> for Nullable<{rs_impl_type}> {{
                                fn from(val: {nullable_type}) -> Self {{
                                    Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                                }}
                            }}

                            impl From<Nullable<{rs_impl_type}>> for {nullable_type} {{
                                fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                                    let val = val.into_value();
                                    let null = val.is_none();
                                    {nullable_type} {{
                                        val: val.unwrap_or({default_val}),
                                        null,
                                    }}
                                }}
                            }}"#,
                            rs_impl_type = rs_impl_type,
                            nullable_type = nullable_type,
                            default_val = default_val,
                        };

                        e.insert([default_impl, nullable_impl].join("\n\n"));
                    }
                }
            }

            if let nullable_type @ TypeAnnotation::Nullable(type_annotation) = &method_spec.ret_type
            {
                let rs_type = type_annotation.as_rs_type()?.0;

                if let std::collections::btree_map::Entry::Vacant(e) = type_impls.entry(rs_type) {
                    let nullable_type = nullable_type.as_rs_bridge_type()?.0;
                    let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                    let default_val = type_annotation.as_rs_default_val()?;

                    let default_impl = formatdoc! {
                        r#"
                        impl Default for {nullable_type} {{
                            fn default() -> Self {{
                                {nullable_type} {{
                                    null: true,
                                    val: {default_val},
                                }}
                            }}
                        }}"#,
                        nullable_type = nullable_type,
                        default_val = default_val,
                    };

                    let nullable_impl = formatdoc! {
                        r#"
                        impl From<{nullable_type}> for Nullable<{rs_impl_type}> {{
                            fn from(val: {nullable_type}) -> Self {{
                                Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                            }}
                        }}

                        impl From<Nullable<{rs_impl_type}>> for {nullable_type} {{
                            fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                                let val = val.into_value();
                                let null = val.is_none();
                                {nullable_type} {{
                                    val: val.unwrap_or({default_val}),
                                    null,
                                }}
                            }}
                        }}"#,
                        rs_impl_type = rs_impl_type,
                        nullable_type = nullable_type,
                        default_val = default_val,
                    };

                    e.insert([default_impl, nullable_impl].join("\n\n"));
                }
            }
        }

        // impl Default trait for the alias type
        for type_annotation in &self.aliases {
            let obj = type_annotation.as_object().unwrap();
            if !type_impls.contains_key(&obj.name) {
                type_impls.insert(obj.name.clone(), alias_default_impl(obj)?);
            }
        }

        for type_annotation in &self.enums {
            let enum_schema = type_annotation.as_enum().unwrap();
            if !type_impls.contains_key(&enum_schema.name) {
                type_impls.insert(enum_schema.name.clone(), enum_default_impl(enum_schema)?);
            }
        }

        Ok(())
    }
}

pub mod template {
    use craby_common::utils::string::snake_case;
    use indoc::formatdoc;

    use crate::{
        parser::types::{EnumTypeAnnotation, ObjectTypeAnnotation, TypeAnnotation},
        utils::indent_str,
    };

    /// Generates Rust struct definition for FFI.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// struct MyStruct {
    ///     foo: String,
    ///     bar: f64,
    ///     baz: bool,
    /// }
    /// ```
    pub fn as_struct_def(obj: &ObjectTypeAnnotation) -> Result<String, anyhow::Error> {
        let mut struct_defs = vec![];
        let mut props = Vec::with_capacity(obj.props.len());

        for prop in &obj.props {
            // Example:
            // ```
            // foo: String,
            // bar: f64,
            // baz: bool,
            // ```
            props.push(format!(
                "{}: {},",
                snake_case(&prop.name),
                prop.type_annotation.as_rs_bridge_type()?.0
            ));

            if let nullable_type @ TypeAnnotation::Nullable(type_annotation) = &prop.type_annotation
            {
                let name = nullable_type.as_rs_bridge_type()?.0;
                let rs_type = type_annotation.as_rs_bridge_type()?.0;
                let struct_def = formatdoc! {
                    r#"
                    struct {name} {{
                        null: bool,
                        val: {rs_type}
                    }}"#,
                    name = name,
                    rs_type = rs_type,
                };

                struct_defs.push(struct_def);
            }
        }

        let struct_def = formatdoc! {
            r#"
            struct {name} {{
            {props}
            }}"#,
            name = obj.name,
            props = indent_str(props.join("\n"), 4),
        };

        struct_defs.push(struct_def);

        Ok(struct_defs.join("\n\n"))
    }

    /// Generates Default implementation for struct types.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// impl Default for MyStruct {
    ///     fn default() -> Self {
    ///         MyStruct {
    ///             foo: String::default(),
    ///             bar: 0.0,
    ///             baz: false,
    ///         }
    ///     }
    /// }
    /// ```
    pub fn alias_default_impl(obj: &ObjectTypeAnnotation) -> Result<String, anyhow::Error> {
        let mut default_impls = vec![];
        let mut props_with_default_val = Vec::with_capacity(obj.props.len());

        for prop in &obj.props {
            props_with_default_val.push(format!(
                "{}: {}",
                snake_case(&prop.name),
                prop.type_annotation.as_rs_default_val()?
            ));

            if let nullable_type @ TypeAnnotation::Nullable(type_annotation) = &prop.type_annotation
            {
                let nullable_type = nullable_type.as_rs_bridge_type()?.0;
                let rs_impl_type = type_annotation.as_rs_impl_type()?.0;
                let default_val = type_annotation.as_rs_default_val()?;

                let default_impl = formatdoc! {
                    r#"
                    impl Default for {nullable_type} {{
                        fn default() -> Self {{
                            {nullable_type} {{
                                null: true,
                                val: {default_val},
                            }}
                        }}
                    }}"#,
                    nullable_type = nullable_type,
                    default_val = default_val,
                };

                let nullable_impl = formatdoc! {
                    r#"
                    impl From<{nullable_type}> for Nullable<{rs_impl_type}> {{
                        fn from(val: {nullable_type}) -> Self {{
                            Nullable::new(if val.null {{ None }} else {{ Some(val.val) }})
                        }}
                    }}

                    impl From<Nullable<{rs_impl_type}>> for {nullable_type} {{
                        fn from(val: Nullable<{rs_impl_type}>) -> Self {{
                            let val = val.into_value();
                            let null = val.is_none();
                            {nullable_type} {{
                                val: val.unwrap_or({default_val}),
                                null,
                            }}
                        }}
                    }}"#,
                    rs_impl_type = rs_impl_type,
                    nullable_type = nullable_type,
                    default_val = default_val,
                };

                default_impls.push(default_impl);
                default_impls.push(nullable_impl);
            }
        }

        let default_impl = formatdoc! {
            r#"
            impl Default for {name} {{
                fn default() -> Self {{
                    {name} {{
            {props}
                    }}
                }}
            }}"#,
            name = obj.name,
            props = indent_str(props_with_default_val.join(",\n"), 12),
        };

        default_impls.push(default_impl);

        Ok(default_impls.join("\n\n"))
    }

    /// Generates Default implementation for enum types.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// impl Default for MyEnum {
    ///     fn default() -> Self {
    ///         MyEnum::FirstMember
    ///     }
    /// }
    /// ```
    pub fn enum_default_impl(enum_schema: &EnumTypeAnnotation) -> Result<String, anyhow::Error> {
        let first_member = enum_schema
            .members
            .first()
            .expect("Enum members are required")
            .name
            .clone();

        let name = enum_schema.name.clone();

        Ok(formatdoc! {
            r#"
            impl Default for {name} {{
                fn default() -> Self {{
                    {name}::{first_member}
                }}
            }}"#,
            name = name,
            first_member = first_member
        })
    }
}
