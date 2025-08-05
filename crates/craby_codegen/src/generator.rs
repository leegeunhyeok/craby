use craby_common::{constants::IMPL_MOD_NAME, utils::sanitize_str};

use crate::types::schema::Schema;

pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_module(&self, schema: &Schema) -> String {
        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.to_rs_fn(4, true))
            .collect::<Vec<_>>();
        let mod_name = sanitize_str(&schema.module_name);

        format!(
            "pub mod {} {{\n    use crate::{};\n\n{}\n}}",
            mod_name,
            IMPL_MOD_NAME,
            methods.join("\n\n")
        )
    }

    pub fn generate_android_ffi_module(
        &self,
        schema: &Schema,
        lib_name: &String,
        java_package_name: &String,
    ) -> String {
        let mod_name = sanitize_str(&schema.module_name);
        let class_name = format!("{}Module", &schema.module_name);
        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.to_android_ffi_fn(lib_name, &mod_name, java_package_name, &class_name))
            .collect::<Vec<_>>();

        format!("use jni::sys::*;\n\n{}", methods.join("\n\n"))
    }

    pub fn generate_ios_ffi_module(&self, schema: &Schema, lib_name: &String) -> String {
        let mod_name = sanitize_str(&schema.module_name);
        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.to_ios_ffi_fn(lib_name, &mod_name))
            .collect::<Vec<_>>();

        methods.join("\n\n")
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
                "    use crate::impls;",
                "",
                "    pub fn multiply(a: f64, b: f64) -> f64 {",
                "        impls::multiply(a, b)",
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

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_module(&schema);

        assert_eq!(
            result,
            [
                "pub mod my_module {",
                "    use crate::impls;",
                "",
                "    pub fn log_message(message: String) {",
                "        impls::log_message(message)",
                "    }",
                "}",
            ]
            .join("\n")
        );
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

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_module(&schema);

        assert_eq!(
            result,
            [
                "pub mod my_module {",
                "    use crate::impls;",
                "",
                "    pub fn greet(name: String, age: Option<f64>) -> String {",
                "        impls::greet(name, age)",
                "    }",
                "}",
            ]
            .join("\n")
        );
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

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_module(&schema);

        assert_eq!(
            result,
            [
                "pub mod my_module {",
                "    use crate::impls;",
                "",
                "    pub fn handle_value(enum_param: String, union_param: f64) {",
                "        impls::handle_value(enum_param, union_param)",
                "    }",
                "}",
            ]
            .join("\n")
        );
    }

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

        let generator = CodeGenerator::new();
        let schema = serde_json::from_str::<Schema>(json_schema).unwrap();
        let result = generator.generate_module(&schema);

        assert_eq!(
            result,
            [
                "pub mod my_module {",
                "    use crate::impls;",
                "",
                "    pub fn nullable_test(nullable_param: Option<f64>) -> Option<String> {",
                "        impls::nullable_test(nullable_param)",
                "    }",
                "}",
            ]
            .join("\n")
        );
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
                "    use crate::impls;",
                "",
                "    pub fn multiply(a: f64, b: f64) -> f64 {",
                "        impls::multiply(a, b)",
                "    }",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_generate_android_ffi_module() {
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
        let result = generator.generate_android_ffi_module(
            &schema,
            &"lib".to_string(),
            &"com.example".to_string(),
        );

        assert_eq!(
            result,
            [
                "use jni::sys::*;",
                "",
                "#[no_mangle]",
                "pub extern \"C\" fn Java_com_example_MyModuleModule_nativeMultiply(_env: JNIEnv, _class: jobject, a: f64, b: f64) -> f64 {",
                "    lib::my_module::multiply(a, b)",
                "}",
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_generate_ios_ffi_module() {
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
        let result = generator.generate_ios_ffi_module(&schema, &"lib".to_string());

        assert_eq!(
            result,
            [
                "#[no_mangle]",
                "pub extern \"C\" fn multiply(a: f64, b: f64) -> f64 {",
                "    lib::my_module::multiply(a, b)",
                "}",
            ]
            .join("\n")
        );
    }
}
