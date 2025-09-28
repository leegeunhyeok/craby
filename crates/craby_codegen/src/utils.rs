use std::collections::{BTreeMap, BTreeSet};

use crate::{
    parser::types::{EnumTypeAnnotation, ObjectTypeAnnotation, TypeAnnotation},
    types::Schema,
};

pub fn indent_str(str: String, indent_size: usize) -> String {
    let indent_str = " ".repeat(indent_size);
    str.lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent_str, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn calc_deps_order(schema: &Schema) -> Result<Vec<String>, anyhow::Error> {
    let mut dependencies = BTreeMap::new();
    let mut visited = BTreeSet::new();
    let mut in_progress = BTreeSet::new();
    let mut result = vec![];

    schema.aliases.iter().for_each(|type_annotation| {
        dependencies.insert(type_annotation.as_object().unwrap().name.clone(), vec![]);
    });

    schema
        .aliases
        .iter()
        .try_for_each(|type_annotation| -> Result<(), anyhow::Error> {
            let alias_spec = type_annotation.as_object().unwrap();

            alias_spec
                .props
                .iter()
                .try_for_each(|prop| -> Result<(), anyhow::Error> {
                    match &prop.type_annotation {
                        TypeAnnotation::Object(ObjectTypeAnnotation {
                            name: alias_name, ..
                        }) => {
                            dependencies
                                .get_mut(&alias_spec.name)
                                .unwrap()
                                .push(alias_name.clone());
                        }
                        TypeAnnotation::Enum(EnumTypeAnnotation {
                            name: enum_name, ..
                        }) => {
                            dependencies
                                .get_mut(&alias_spec.name)
                                .unwrap()
                                .push(enum_name.clone());
                        }
                        nullable @ TypeAnnotation::Nullable(type_annotation) => {
                            let rs_type = nullable.as_rs_bridge_type()?.0;
                            dependencies.entry(rs_type.clone()).or_insert(vec![]);

                            match &**type_annotation {
                                TypeAnnotation::Object(ObjectTypeAnnotation {
                                    name: alias_name,
                                    ..
                                }) => {
                                    dependencies
                                        .get_mut(&rs_type)
                                        .unwrap()
                                        .push(alias_name.clone());
                                }
                                TypeAnnotation::Enum(EnumTypeAnnotation {
                                    name: enum_name,
                                    ..
                                }) => {
                                    dependencies
                                        .get_mut(&rs_type)
                                        .unwrap()
                                        .push(enum_name.clone());
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                    Ok(())
                })?;

            Ok(())
        })?;

    fn visit(
        node: &str,
        dependencies: &BTreeMap<String, Vec<String>>,
        visited: &mut BTreeSet<String>,
        in_progress: &mut BTreeSet<String>,
        result: &mut Vec<String>,
    ) -> Result<(), anyhow::Error> {
        if in_progress.contains(node) {
            return Err(anyhow::anyhow!(
                "Circular dependency detected involving: {}",
                node
            ));
        }

        if visited.contains(node) {
            return Ok(());
        }

        in_progress.insert(node.to_string());

        if let Some(deps) = dependencies.get(node) {
            for dep in deps {
                visit(dep, dependencies, visited, in_progress, result)?;
            }
        }

        in_progress.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());

        Ok(())
    }

    for node in dependencies.keys() {
        if !visited.contains(node) {
            visit(
                node,
                &dependencies,
                &mut visited,
                &mut in_progress,
                &mut result,
            )?;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_str() {
        assert_eq!(
            indent_str("Hello\nWorld".to_string(), 2),
            "  Hello\n  World"
        );
        assert_eq!(
            indent_str("Hello\nWorld".to_string(), 4),
            "    Hello\n    World"
        );
    }
}
