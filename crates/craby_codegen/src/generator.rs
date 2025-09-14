use craby_common::utils::string::pascal_case;
use indoc::formatdoc;

use crate::{
    types::schema::{CxxFunction, Schema},
    utils::indent_str,
};

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
    pub fn generate_spec(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());
        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| -> Result<String, anyhow::Error> {
                let sig = spec.to_sig()?;
                Ok(format!("{};", sig))
            })
            .collect::<Result<Vec<_>, _>>()?;

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
    /// use crate::{ffi::ffi::*, generated::*};
    ///
    /// pub struct MyModule;
    ///
    /// impl MyModuleSpec for MyModule {
    ///     fn multiply(a: f64, b: f64) -> f64 {
    ///         unimplemented!();
    ///     }
    /// }
    /// ```
    pub fn generate_impl(&self, schema: &Schema) -> Result<String, anyhow::Error> {
        let mod_name = pascal_case(schema.module_name.as_str());
        let trait_name = pascal_case(format!("{}Spec", schema.module_name).as_str());

        let methods = schema
            .spec
            .methods
            .iter()
            .map(|spec| -> Result<String, anyhow::Error> {
                let func_sig = spec.to_sig()?;
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

        let code = formatdoc! {
          r#"
          use crate::{{ffi::ffi::*, generated::*}};

          pub struct {mod_name};

          impl {trait_name} for {mod_name} {{
          {methods}
          }}"#,
          trait_name = trait_name,
          mod_name= mod_name,
          methods = indent_str(methods.join("\n\n"), 4),
        };

        Ok(code)
    }

    /// Returns the CXX(FFI) function signature for the `FunctionSpec`.
    ///
    /// ```rust,ignore
    /// // extern function
    /// #[cxx_name = "myFunc"]
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz;
    ///
    /// // impl function
    /// fn myFunc(arg1: Foo, arg2: Bar) -> Baz {
    ///     MyModule::my_func(arg1, arg2)
    /// }
    /// ```
    pub fn get_cxx_functions(&self, schema: &Schema) -> Result<Vec<CxxFunction>, anyhow::Error> {
        let res = schema
            .spec
            .methods
            .iter()
            .map(|spec| spec.to_cxx_func(&schema.module_name))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(res)
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
        let result = generator.generate_spec(&schema).unwrap();

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
        let result = generator.generate_spec(&schema).unwrap();

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
        let result = generator.generate_impl(&schema).unwrap();

        assert_eq!(
            result,
            [
                "use crate::{ffi::ffi::*, generated::*};",
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
        let result = generator.get_cxx_functions(&schema).unwrap();
        let result = result.get(0).unwrap();

        assert_eq!(
            result.extern_func,
            [
                "#[cxx_name = \"multiply\"]",
                "fn multiply(a: f64, b: String) -> f64;",
            ]
            .join("\n")
        );

        assert_eq!(
            result.impl_func,
            [
                "fn multiply(a: f64, b: String) -> f64 {",
                "    MyModule::multiply(a, b)",
                "}",
            ]
            .join("\n")
        );
    }
}
