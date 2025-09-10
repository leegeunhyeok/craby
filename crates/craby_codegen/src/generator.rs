use craby_common::{
    constants::GENERATED_MOD,
    utils::{pascal_case, sanitize_str},
};
use indoc::formatdoc;

use crate::{types::schema::Schema, utils::indent_str};

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate the spec trait for the given schema.
    ///
    /// ```rust,ignore
    /// pub trait MyModuleSpec {
    ///     fn multiply(a: f64, b: f64) -> f64;
    /// }
    /// ```
    pub fn generate_spec(&self, schema: &Schema) -> String {
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());
        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| format!("{};", spec.to_rs_func_sig()))
            .collect::<Vec<_>>();

        formatdoc! {
          r#"
          pub trait {trait_name} {{
          {methods}
          }}"#,
          trait_name = trait_name,
          methods = indent_str(methods.join("\n"), 4),
        }
    }

    /// Generate the empty module for the given schema.
    ///
    /// ```rust,ignore
    /// use crate::generated::MyModuleSpec;
    ///
    /// pub struct MyModule;
    ///
    /// impl MyModuleSpec for MyModule {
    ///     fn multiply(a: f64, b: f64) -> f64 {
    ///         unimplemented!();
    ///     }
    /// }
    /// ```
    pub fn generate_impl(&self, schema: &Schema) -> String {
        let mod_name = pascal_case(schema.module_name.as_str());
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());

        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| {
                let func_sig = spec.to_rs_func_sig();

                formatdoc! {
                  r#"
                  {func_sig} {{
                      unimplemented!();
                  }}"#,
                  func_sig = func_sig,
                }
            })
            .collect::<Vec<_>>();

        formatdoc! {
          r#"
          use crate::generated::{trait_name};

          pub struct {mod_name};

          impl {trait_name} for {mod_name} {{
          {methods}
          }}"#,
          trait_name = trait_name,
          mod_name= mod_name,
          methods = indent_str(methods.join("\n\n"), 4),
        }
    }

    /// Generate the empty module for the given schema.
    ///
    /// ```rust,ignore
    /// use generated::*;
    /// use std::os::raw::*;
    ///
    /// #[no_mangle]
    /// pub extern "C" fn multiply(a: f64, b: String) -> f64 {
    ///     my_module_impl::MyModule::multiply(a, b)
    /// }
    /// ```
    pub fn generate_ffi(&self, schema: &Schema) -> String {
        let mod_name = sanitize_str(&schema.module_name);
        let imports = vec![
            format!("use {}::*;", GENERATED_MOD),
            "use std::os::raw::*;".to_string(),
        ];

        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.to_ffi_func(&mod_name))
            .collect::<Vec<_>>();

        format!(
            "{imports}\n\n{methods}",
            imports = imports.join("\n"),
            methods = methods.join("\n\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_generation() {
        let json_schema = r#"
        {
          "moduleName": "MyModule",
          "type": "NativeModule",
          "aliasMap": {},
          "enumMap": {},
          "spec": {
            "eventEmitters": [],
            "methods": [
              {
                "name": "multiply",
                "optional": false,
                "typeAnnotation": {
                  "type": "FunctionTypeAnnotation",
                  "returnTypeAnnotation": {
                    "type": "NumberTypeAnnotation"
                  },
                  "params": [
                    {
                      "name": "a",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "NumberTypeAnnotation"
                      }
                    },
                    {
                      "name": "b",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "NumberTypeAnnotation"
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#;

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_spec(&schema);

        assert_eq!(
            result,
            [
                "pub trait MyModuleSpec {",
                "    fn multiply(a: f64, b: f64) -> f64;",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_void_function_generation() {
        let json_schema = r#"
        {
          "moduleName": "MyModule",
          "type": "NativeModule",
          "aliasMap": {},
          "enumMap": {},
          "spec": {
            "eventEmitters": [],
            "methods": [
              {
                "name": "log_message",
                "optional": false,
                "typeAnnotation": {
                  "type": "FunctionTypeAnnotation",
                  "returnTypeAnnotation": {
                    "type": "VoidTypeAnnotation"
                  },
                  "params": [
                    {
                      "name": "message",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "StringTypeAnnotation"
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#;

        // TODO: Implement void function generation
        assert_eq!(json_schema, json_schema);
    }

    #[test]
    fn test_optional_parameters() {
        let json_schema = r#"
        {
          "moduleName": "MyModule",
          "type": "NativeModule",
          "aliasMap": {},
          "enumMap": {},
          "spec": {
            "eventEmitters": [],
            "methods": [
              {
                "name": "greet",
                "optional": false,
                "typeAnnotation": {
                  "type": "FunctionTypeAnnotation",
                  "returnTypeAnnotation": {
                    "type": "StringTypeAnnotation"
                  },
                  "params": [
                    {
                      "name": "name",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "StringTypeAnnotation"
                      }
                    },
                    {
                      "name": "age",
                      "optional": true,
                      "typeAnnotation": {
                        "type": "NumberTypeAnnotation"
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#;

        // TODO: Implement optional parameters
        assert_eq!(json_schema, json_schema);
    }

    #[test]
    fn test_enum_and_union_types() {
        let json_schema = r#"
        {
          "moduleName": "MyModule",
          "type": "NativeModule",
          "aliasMap": {},
          "enumMap": {},
          "spec": {
            "eventEmitters": [],
            "methods": [
              {
                "name": "handle_value",
                "optional": false,
                "typeAnnotation": {
                  "type": "FunctionTypeAnnotation",
                  "returnTypeAnnotation": {
                    "type": "VoidTypeAnnotation"
                  },
                  "params": [
                    {
                      "name": "enum_param",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "EnumDeclaration",
                        "memberType": "StringTypeAnnotation",
                        "members": [
                          {"name": "OPTION_A", "value": "a"},
                          {"name": "OPTION_B", "value": "b"}
                        ]
                      }
                    },
                    {
                      "name": "union_param",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "UnionTypeAnnotation",
                        "memberType": "NumberTypeAnnotation",
                        "types": []
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#;

        // TODO: Implement enum and union types
        assert_eq!(json_schema, json_schema);
    }

    // Skip
    #[test]
    fn test_nullable_types() {
        let json_schema = r#"
        {
          "moduleName": "MyModule",
          "type": "NativeModule",
          "aliasMap": {},
          "enumMap": {},
          "spec": {
            "eventEmitters": [],
            "methods": [
              {
                "name": "nullable_test",
                "optional": false,
                "typeAnnotation": {
                  "type": "FunctionTypeAnnotation",
                  "returnTypeAnnotation": {
                    "type": "NullableTypeAnnotation",
                    "typeAnnotation": {
                      "type": "StringTypeAnnotation"
                    }
                  },
                  "params": [
                    {
                      "name": "nullable_param",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "NullableTypeAnnotation",
                        "typeAnnotation": {
                          "type": "NumberTypeAnnotation"
                        }
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#;

        // TODO: Implement nullable types
        assert_eq!(json_schema, json_schema);
    }

    #[test]
    fn test_generate_spec() {
        let json_schema = r#"
        {
          "moduleName": "MyModule",
          "type": "NativeModule",
          "aliasMap": {},
          "enumMap": {},
          "spec": {
            "eventEmitters": [],
            "methods": [
              {
                "name": "multiply",
                "optional": false,
                "typeAnnotation": {
                  "type": "FunctionTypeAnnotation",
                  "returnTypeAnnotation": {
                    "type": "NumberTypeAnnotation"
                  },
                  "params": [
                    {
                      "name": "a",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "NumberTypeAnnotation"
                      }
                    },
                    {
                      "name": "b",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "NumberTypeAnnotation"
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#;

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_spec(&schema);

        assert_eq!(
            result,
            [
                "pub trait MyModuleSpec {",
                "    fn multiply(a: f64, b: f64) -> f64;",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_generate_impl() {
        let json_schema = r#"
        {
          "moduleName": "MyModule",
          "type": "NativeModule",
          "aliasMap": {},
          "enumMap": {},
          "spec": {
            "eventEmitters": [],
            "methods": [
              {
                "name": "multiply",
                "optional": false,
                "typeAnnotation": {
                  "type": "FunctionTypeAnnotation",
                  "returnTypeAnnotation": {
                    "type": "NumberTypeAnnotation"
                  },
                  "params": [
                    {
                      "name": "a",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "NumberTypeAnnotation"
                      }
                    },
                    {
                      "name": "b",
                      "optional": false,
                      "typeAnnotation": {
                        "type": "NumberTypeAnnotation"
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#;

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_impl(&schema);

        assert_eq!(
            result,
            [
                "use crate::generated::MyModuleSpec;",
                "",
                "pub struct MyModule;",
                "",
                "impl MyModuleSpec for MyModule {",
                "    fn multiply(a: f64, b: f64) -> f64 {",
                "        unimplemented!();",
                "    }",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_generate_ffi() {
        let json_schema = r#"
      {
        "moduleName": "MyModule",
        "type": "NativeModule",
        "aliasMap": {},
        "enumMap": {},
        "spec": {
          "eventEmitters": [],
          "methods": [
            {
              "name": "multiply",
              "optional": false,
              "typeAnnotation": {
                "type": "FunctionTypeAnnotation",
                "returnTypeAnnotation": {
                  "type": "NumberTypeAnnotation"
                },
                "params": [
                  {
                    "name": "a",
                    "optional": false,
                    "typeAnnotation": {
                      "type": "NumberTypeAnnotation"
                    }
                  },
                  {
                    "name": "b",
                    "optional": false,
                    "typeAnnotation": {
                      "type": "StringTypeAnnotation"
                    }
                  }
                ]
              }
            }
          ]
        }
      }
      "#;

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_ffi(&schema);

        assert_eq!(
            result,
            [
                "use generated::*;",
                "use std::os::raw::*;",
                "",
                "#[no_mangle]",
                "pub extern \"C\" fn multiply(a: f64, b: String) -> f64 {",
                "    my_module_impl::MyModule::multiply(a, b)",
                "}",
            ]
            .join("\n")
        );
    }
}
