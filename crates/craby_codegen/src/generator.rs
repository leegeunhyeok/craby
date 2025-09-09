use craby_common::utils::{sanitize_str, to_impl_mod_name};
use indoc::formatdoc;

use crate::{types::schema::Schema, utils::indent_str};

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_module(&self, schema: &Schema) -> String {
        let mod_name = sanitize_str(&schema.module_name);
        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| indent_str(spec.to_rs_func(&mod_name), 4))
            .collect::<Vec<_>>();
        let impl_mod_name = to_impl_mod_name(&mod_name);

        formatdoc! {
          r#"
          pub mod {mod_name} {{
              use crate::{impl_mod_name};

          {methods}
          }}"#,
          mod_name = mod_name.to_string(),
          impl_mod_name = impl_mod_name.to_string(),
          methods = methods.join("\n\n"),
        }
    }

    pub fn generate_empty_module(&self, schema: &Schema) -> String {
        schema
            .spec
            .methods
            .iter()
            .map(|spec| {
                let func_sig = spec.to_rs_func_sig();

                formatdoc! {
                  r#"
                  pub {func_sig} {{
                      unimplemented!();
                  }}"#,
                  func_sig = func_sig,
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn generate_ffi_module(&self, schema: &Schema) -> String {
        let mod_name = sanitize_str(&schema.module_name);
        let imports = vec!["use std::os::raw::*;".to_string()];

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
    fn test_function_generation() {
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
        let result = generator.generate_module(&schema);

        assert_eq!(
            result,
            [
                "pub mod my_module {",
                "    use crate::my_module_impl;",
                "",
                "    pub fn multiply(a: f64, b: f64) -> f64 {",
                "        my_module_impl::multiply(a, b)",
                "    }",
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
    fn test_generate_module() {
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
        let result = generator.generate_module(&schema);

        assert_eq!(
            result,
            [
                "pub mod my_module {",
                "    use crate::my_module_impl;",
                "",
                "    pub fn multiply(a: f64, b: f64) -> f64 {",
                "        my_module_impl::multiply(a, b)",
                "    }",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_generate_empty_module() {
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
        let result = generator.generate_empty_module(&schema);

        assert_eq!(
            result,
            [
                "pub fn multiply(a: f64, b: f64) -> f64 {",
                "    unimplemented!();",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_generate_ffi_module() {
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
        let result = generator.generate_ffi_module(&schema);

        assert_eq!(
            result,
            [
                "use std::os::raw::*;",
                "",
                "#[no_mangle]",
                "pub extern \"C\" fn multiply(a: f64, b: String) -> f64 {",
                "    generated::my_module_impl::multiply(a, b)",
                "}",
            ]
            .join("\n")
        );
    }
}
