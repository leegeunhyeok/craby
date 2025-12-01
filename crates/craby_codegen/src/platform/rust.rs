use std::collections::{
    btree_map::Entry as BTreeMapEntry, hash_map::Entry as HashMapEntry, BTreeMap,
};

use craby_common::utils::string::{camel_case, pascal_case, snake_case};
use indoc::formatdoc;
use rustc_hash::FxHashMap;

use crate::{
    common::IntoCode,
    constants::specs::RESERVED_ARG_NAME_MODULE,
    parser::types::{
        EnumTypeAnnotation, Method, ObjectTypeAnnotation, Param, RefTypeAnnotation, TypeAnnotation,
    },
    platform::rust::template::{
        collect_alias_default_impls, RsDefaultImpl, RsNullableStruct, RsStruct,
    },
    types::Schema,
    utils::indent_str,
};

#[derive(Debug)]
pub struct RsType(String);

impl IntoCode for RsType {
    fn into_code(self) -> String {
        self.0
    }
}

#[derive(Debug)]
pub struct RsBridgeType(String);

impl IntoCode for RsBridgeType {
    fn into_code(self) -> String {
        self.0
    }
}

#[derive(Debug)]
pub struct RsImplType(pub String);

impl IntoCode for RsImplType {
    fn into_code(self) -> String {
        self.0
    }
}

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
    /// fn my_func(arg1: Foo, arg2: Bar) -> Result<Baz>;
    /// ```
    pub func_extern_sigs: Vec<String>,
    /// The implementation function of the extern function.
    ///
    /// **Example**
    ///
    /// ```rust,ignore
    /// fn my_func(arg1: Foo, arg2: Bar) -> Result<Baz> {
    ///     craby::catch_panic!({
    ///         let ret = it_.my_func(arg1, arg2);
    ///         ret
    ///     })
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
            TypeAnnotation::ArrayBuffer => "Vec<u8>".to_string(),
            TypeAnnotation::Array(element_type) => {
                if let TypeAnnotation::Array(..) = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Vec<{}>", element_type.as_rs_type()?.into_code())
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Promise(resolve_type) => {
                format!(
                    "Result<{}, anyhow::Error>",
                    resolve_type.as_rs_type()?.into_code()
                )
            }
            TypeAnnotation::Nullable(type_annotation) => match &**type_annotation {
                TypeAnnotation::Boolean => "NullableBoolean".to_string(),
                TypeAnnotation::Number => "NullableNumber".to_string(),
                TypeAnnotation::String => "NullableString".to_string(),
                TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                    format!("Nullable{name}")
                }
                TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                    format!("Nullable{name}")
                }
                TypeAnnotation::Ref(RefTypeAnnotation { name, .. }) => {
                    format!("Nullable{name}")
                }
                TypeAnnotation::ArrayBuffer => "NullableArrayBuffer".to_string(),
                TypeAnnotation::Array(element_type) => match &**element_type {
                    TypeAnnotation::Boolean => "NullableBooleanArray".to_string(),
                    TypeAnnotation::Number => "NullableNumberArray".to_string(),
                    TypeAnnotation::String => "NullableStringArray".to_string(),
                    TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                        format!("Nullable{name}Array")
                    }
                    TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                        format!("Nullable{name}Array")
                    }
                    TypeAnnotation::Ref(RefTypeAnnotation { name, .. }) => {
                        format!("Nullable{name}Array")
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
                format!("Result<{}>", resolve_type.as_rs_type()?.into_code())
            }
            _ => self.as_rs_type()?.into_code(),
        };

        Ok(RsBridgeType(extern_type))
    }

    /// Converts TypeAnnotation to user-facing Rust implementation type.
    ///
    /// # Generated Code Examples
    ///
    /// ```rust,ignore
    /// Boolean          // Boolean (aliased bool)
    /// Number           // Number (aliased f64)
    /// String           // String
    /// ArrayBuffer      // ArrayBuffer (aliased Vec<u8>)
    /// Array<Number>    // Array<Number>
    /// Promise<Number>  // Promise<Number>
    /// Nullable<Number> // Nullable<Number>
    /// ```
    pub fn as_rs_impl_type(&self) -> Result<RsImplType, anyhow::Error> {
        let rs_type = match self {
            TypeAnnotation::Void => "Void".to_string(),
            TypeAnnotation::Boolean => "Boolean".to_string(),
            TypeAnnotation::Number => "Number".to_string(),
            TypeAnnotation::String => "String".to_string(),
            TypeAnnotation::ArrayBuffer => "ArrayBuffer".to_string(),
            TypeAnnotation::Array(element_type) => {
                if let TypeAnnotation::Array { .. } = &**element_type {
                    return Err(anyhow::anyhow!(
                        "Nested array type is not supported: {:?}",
                        element_type
                    ));
                }
                format!("Array<{}>", element_type.as_rs_impl_type()?.into_code())
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => name.clone(),
            TypeAnnotation::Promise(resolved_type) => {
                format!("Promise<{}>", resolved_type.as_rs_impl_type()?.into_code())
            }
            TypeAnnotation::Nullable(type_annotation) => {
                let type_annotation = type_annotation.as_rs_impl_type()?.into_code();
                format!("Nullable<{type_annotation}>")
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
            TypeAnnotation::ArrayBuffer | TypeAnnotation::Array(..) => "Vec::default()".to_string(),
            TypeAnnotation::Enum(EnumTypeAnnotation { name, .. }) => {
                format!("{name}::default()")
            }
            TypeAnnotation::Object(ObjectTypeAnnotation { name, .. }) => {
                format!("{name}::default()")
            }
            TypeAnnotation::Nullable(..) => {
                let nullable_type = self.as_rs_type()?.into_code();
                format!("{nullable_type}::default()")
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
        let return_type = self.ret_type.as_rs_impl_type()?.into_code();
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
            format!(" -> {return_type}")
        };

        Ok(format!("fn {fn_name}({params_sig}){ret_annotation}"))
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
        let param_type = if let TypeAnnotation::String = &self.type_annotation {
            "&str".to_string()
        } else {
            self.type_annotation.as_rs_type()?.into_code()
        };
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
        let param_type = if let TypeAnnotation::String = &self.type_annotation {
            "&str".to_string()
        } else {
            self.type_annotation.as_rs_impl_type()?.into_code()
        };
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
    /// fn create_my_module(id: usize, data_path: &str) -> Box<MyModule>;
    ///
    /// #[cxx_name = "multiply"]
    /// fn my_module_multiply(it_: &mut MyModule, a: f64, b: f64) -> Result<f64>;
    ///
    /// // Implementation:
    /// fn create_my_module(id: usize, data_path: &str) -> Box<MyModule> {
    ///     Box::new(MyModule::new(id, data_path))
    /// }
    ///
    /// fn my_module_multiply(it_: &mut MyModule, a: f64, b: f64) -> Result<f64> {
    ///     craby::catch_panic!({
    ///         let ret = it_.multiply(a, b);
    ///         ret
    ///     })
    /// }
    /// ```
    pub fn as_rs_cxx_bridge(&self) -> Result<RsCxxBridge, anyhow::Error> {
        let module_name = pascal_case(&self.module_name);
        let snake_module_name = snake_case(&self.module_name);

        let mut func_extern_sigs = Vec::with_capacity(self.methods.len() + 1);
        let mut func_impls = Vec::with_capacity(self.methods.len() + 1);
        let mut type_impls = vec![];
        let mut struct_defs = FxHashMap::default();

        func_extern_sigs.push(formatdoc! {
            r#"
            #[cxx_name = "create{module_name}"]
            fn create_{snake_module_name}(id: usize, data_path: &str) -> Box<{module_name}>;"#,
        });

        func_impls.push(formatdoc! {
            r#"
            fn create_{snake_module_name}(id: usize, data_path: &str) -> Box<{module_name}> {{
                let ctx = Context::new(id, data_path);
                Box::new({module_name}::new(ctx))
            }}"#,
        });

        // Collect extern function signatures and implementations
        for method_spec in &self.methods {
            // Collect nullable parameters
            for param in &method_spec.params {
                if param.type_annotation.is_nullable() {
                    let id = param.type_annotation.to_id();
                    if let HashMapEntry::Vacant(e) = struct_defs.entry(id) {
                        let nullable = RsNullableStruct::try_from(&param.type_annotation)?;
                        e.insert(nullable.definition);
                        type_impls.push(nullable.implementation);
                    }
                }
            }

            // Collect nullable return type
            if method_spec.ret_type.is_nullable() {
                let id = method_spec.ret_type.to_id();
                if let HashMapEntry::Vacant(e) = struct_defs.entry(id) {
                    let nullable = RsNullableStruct::try_from(&method_spec.ret_type)?;
                    e.insert(nullable.definition);
                    type_impls.push(nullable.implementation);
                }
            }

            let ret_type = method_spec.ret_type.as_rs_type()?.into_code();
            let ret_type = match method_spec.ret_type {
                TypeAnnotation::Promise(_) => ret_type,
                _ => format!("Result<{ret_type}, anyhow::Error>"),
            };
            let ret_extern_type = method_spec.ret_type.as_rs_bridge_type()?.into_code();
            let ret_extern_type = match method_spec.ret_type {
                TypeAnnotation::Promise(_) => ret_extern_type,
                _ => format!("Result<{ret_extern_type}>"),
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
                            "{RESERVED_ARG_NAME_MODULE}: &mut {}",
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
                        format!("{name}.into()")
                    } else {
                        name
                    }
                })
                .collect::<Vec<_>>();

            let cxx_extern_fn_name = camel_case(&method_spec.name);
            let prefixed_fn_name = format!("{mod_name}_{fn_name}");
            let ret_extern_annotation = format!(" -> {ret_extern_type}");
            let ret_annotation = format!(" -> {ret_type}");
            let extern_func = formatdoc! {
                r#"
                #[cxx_name = "{cxx_extern_fn_name}"]
                fn {prefixed_fn_name}({params_sig}){ret_extern_annotation};"#,
            };

            let ret = if let TypeAnnotation::Nullable(..) = &method_spec.ret_type {
                "ret.into()"
            } else {
                "ret"
            };

            let fn_args = fn_args.join(", ");
            let impl_func = match method_spec.ret_type {
                TypeAnnotation::Promise(_) => formatdoc! {
                    r#"
                    fn {prefixed_fn_name}({params_sig}){ret_annotation} {{
                        craby::catch_panic!({{
                            let ret = {it}.{fn_name}({fn_args});
                            {ret}
                        }}).and_then(|r| r)
                    }}"#,
                    it = RESERVED_ARG_NAME_MODULE,
                },
                _ => formatdoc! {
                    r#"
                    fn {prefixed_fn_name}({params_sig}){ret_annotation} {{
                        craby::catch_panic!({{
                            let ret = {it}.{fn_name}({fn_args});
                            {ret}
                        }})
                    }}"#,
                    it = RESERVED_ARG_NAME_MODULE,
                },
            };

            func_extern_sigs.push(extern_func);
            func_impls.push(impl_func);
        }

        // Collect alias types (struct)
        for type_annotation in &self.aliases {
            if let HashMapEntry::Vacant(e) = struct_defs.entry(type_annotation.to_id()) {
                let id = type_annotation.to_id();
                let obj = type_annotation.as_object().unwrap();
                e.insert(RsStruct::try_from(obj)?.into_code());

                for prop in &obj.props {
                    if prop.type_annotation.is_nullable() {
                        let id = prop.type_annotation.to_id();
                        if let HashMapEntry::Vacant(e) = struct_defs.entry(id) {
                            let nullable = RsNullableStruct::try_from(&prop.type_annotation)?;
                            e.insert(nullable.definition);
                        }
                    }
                }

                // Collect default implementations for the alias type
                let mut type_impls_map = BTreeMap::new();
                collect_alias_default_impls(id, obj, &mut type_impls_map)?;

                type_impls.push(
                    type_impls_map
                        .into_values()
                        .collect::<Vec<_>>()
                        .join("\n\n"),
                );
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
                    .collect::<Vec<_>>();

                let members = indent_str(&members.join("\n"), 4);
                formatdoc! {
                    r#"
                    enum {name} {{
                    {members}
                    }}"#,
                    name = enum_schema.name,
                }
            })
            .collect();

        Ok(RsCxxBridge {
            impl_type: format!("type {module_name};"),
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
        type_impls: &mut BTreeMap<u64, String>,
    ) -> Result<(), anyhow::Error> {
        // Collect extern function signatures and implementations
        for method_spec in &self.methods {
            for param in &method_spec.params {
                // Collect nullable parameters
                if param.type_annotation.is_nullable() {
                    let id = param.type_annotation.to_id();
                    if let BTreeMapEntry::Vacant(e) = type_impls.entry(id) {
                        let nullable = RsNullableStruct::try_from(&param.type_annotation)?;
                        e.insert(nullable.implementation);
                    }
                }
            }

            // Collect nullable return type
            if method_spec.ret_type.is_nullable() {
                let id = method_spec.ret_type.to_id();
                if let BTreeMapEntry::Vacant(e) = type_impls.entry(id) {
                    let nullable = RsNullableStruct::try_from(&method_spec.ret_type)?;
                    e.insert(nullable.implementation);
                }
            }
        }

        // impl Default trait for the alias type
        for type_annotation in &self.aliases {
            let id = type_annotation.to_id();
            if !type_impls.contains_key(&id) {
                let obj = type_annotation.as_object().unwrap();
                collect_alias_default_impls(id, obj, type_impls)?;
            }
        }

        for type_annotation in &self.enums {
            let id = type_annotation.to_id();
            if let BTreeMapEntry::Vacant(e) = type_impls.entry(id) {
                let enum_type_annotation = type_annotation.as_enum().unwrap();
                e.insert(RsDefaultImpl::try_from(enum_type_annotation)?.into_code());
            }
        }

        Ok(())
    }
}

pub mod template {
    use std::collections::{btree_map::Entry as BTreeMapEntry, BTreeMap};

    use craby_common::utils::string::snake_case;
    use indoc::formatdoc;

    use crate::{
        common::IntoCode,
        parser::types::{EnumTypeAnnotation, ObjectTypeAnnotation, TypeAnnotation},
        utils::indent_str,
    };

    /// Rust struct definition for FFI.
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
    pub struct RsStruct(pub String);

    impl IntoCode for RsStruct {
        fn into_code(self) -> String {
            self.0
        }
    }

    impl TryFrom<&ObjectTypeAnnotation> for RsStruct {
        type Error = anyhow::Error;

        fn try_from(obj: &ObjectTypeAnnotation) -> Result<Self, Self::Error> {
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
                    prop.type_annotation.as_rs_bridge_type()?.into_code()
                ));
            }

            let props = indent_str(&props.join("\n"), 4);
            let struct_def = formatdoc! {
                r#"
                #[derive(Clone)]
                struct {name} {{
                {props}
                }}"#,
                name = obj.name,
            };

            Ok(RsStruct(struct_def))
        }
    }

    /// Rust struct definition for nullable types.
    pub struct RsNullableStruct {
        pub definition: String,
        pub implementation: String,
    }

    impl TryFrom<&TypeAnnotation> for RsNullableStruct {
        type Error = anyhow::Error;

        fn try_from(nullable_type: &TypeAnnotation) -> Result<Self, Self::Error> {
            if let TypeAnnotation::Nullable(type_annotation) = nullable_type {
                let struct_type = nullable_type.as_rs_bridge_type()?.into_code();
                let base_type = type_annotation.as_rs_type()?.into_code();
                let rs_impl_type = type_annotation.as_rs_impl_type()?.into_code();
                let default_val = type_annotation.as_rs_default_val()?;

                let struct_def = formatdoc! {
                    r#"
                    #[derive(Clone)]
                    struct {struct_type} {{
                        null: bool,
                        val: {base_type},
                    }}"#,
                };

                let struct_impl = formatdoc! {
                    r#"
                    impl Default for {struct_type} {{
                        fn default() -> Self {{
                            {struct_type} {{
                                null: true,
                                val: {default_val},
                            }}
                        }}
                    }}

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
                };

                return Ok(RsNullableStruct {
                    definition: struct_def,
                    implementation: struct_impl,
                });
            }

            anyhow::bail!("Not a nullable type: {:?}", nullable_type);
        }
    }

    /// Default implementation for struct types.
    ///
    /// # Generated Code
    ///
    /// ```rust,ignore
    /// // Struct
    /// impl Default for MyStruct {
    ///     fn default() -> Self {
    ///         MyStruct {
    ///             foo: String::default(),
    ///             bar: 0.0,
    ///             baz: false,
    ///         }
    ///     }
    /// }
    ///
    /// // Enum
    /// impl Default for MyEnum {
    ///     fn default() -> Self {
    ///         MyEnum::FirstMember
    ///     }
    /// }
    /// ```
    pub struct RsDefaultImpl(pub String);

    impl IntoCode for RsDefaultImpl {
        fn into_code(self) -> String {
            self.0
        }
    }

    impl TryFrom<&ObjectTypeAnnotation> for RsDefaultImpl {
        type Error = anyhow::Error;

        fn try_from(obj: &ObjectTypeAnnotation) -> Result<Self, Self::Error> {
            let mut props_with_default_val = Vec::with_capacity(obj.props.len());

            for prop in &obj.props {
                props_with_default_val.push(format!(
                    "{}: {}",
                    snake_case(&prop.name),
                    prop.type_annotation.as_rs_default_val()?
                ));
            }

            let props = indent_str(&props_with_default_val.join(",\n"), 12);
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
            };

            Ok(RsDefaultImpl(default_impl))
        }
    }

    impl TryFrom<&EnumTypeAnnotation> for RsDefaultImpl {
        type Error = anyhow::Error;

        fn try_from(enum_type_annotation: &EnumTypeAnnotation) -> Result<Self, Self::Error> {
            let first_member = enum_type_annotation
                .members
                .first()
                .ok_or_else(|| anyhow::anyhow!("Enum members are required"))?;

            let default_impl = formatdoc! {
                r#"
                impl Default for {name} {{
                    fn default() -> Self {{
                        {name}::{first_member}
                    }}
                }}"#,
                name = enum_type_annotation.name,
                first_member = first_member.name
            };

            Ok(RsDefaultImpl(default_impl))
        }
    }

    pub fn collect_alias_default_impls(
        id: u64,
        obj: &ObjectTypeAnnotation,
        type_impls: &mut BTreeMap<u64, String>,
    ) -> Result<(), anyhow::Error> {
        for prop in &obj.props {
            if prop.type_annotation.is_nullable() {
                let id = prop.type_annotation.to_id();
                if let BTreeMapEntry::Vacant(e) = type_impls.entry(id) {
                    let nullable = RsNullableStruct::try_from(&prop.type_annotation)?;
                    e.insert(nullable.implementation);
                }
            }
        }

        type_impls.insert(id, RsDefaultImpl::try_from(obj)?.into_code());
        Ok(())
    }
}
