use std::{collections::BTreeMap, path::PathBuf};

use log::debug;
use oxc::{
    allocator::Allocator,
    ast::ast::*,
    ast_visit::Visit,
    diagnostics::{GraphicalReportHandler, GraphicalTheme, OxcDiagnostic},
    parser::Parser,
    semantic::{ReferenceId, Scoping, SemanticBuilder, SymbolId},
};

use crate::{
    parser::{
        types::*,
        utils::{error, render_report, RenderReportOptions},
    },
    types::schema::Schema2,
};

const REACT_NATIVE_PKG: &str = "react-native";
const NATIVE_MODULE_INTERFACE: &str = "TurboModule";
const NATIVE_MODULE_REGISTRY: &str = "TurboModuleRegistry";
const REGISTRY_GET: &str = "get";
const REGISTRY_GET_ENFORCING: &str = "getEnforcing";

const INVALID_SPEC: &str = "Invalid specification";
const INVALID_TYPE_REFERENCE: &str = "Invalid type reference";
const INVALID_COMPUTED_SIG: &str = "Computed signature is not supported";
const INVALID_OPTIONAL_SIG: &str = "Optional signature is not supported";
const INVALID_NO_SPEC_GENERIC: &str = "NativeModule specification generic argument is required";
const INVALID_FUNC_PARAM: &str = "Function parameter is not supported";
const INVALID_TYPE_LITERAL: &str =
    "Type literal is not supported. Use defined type reference instead";
const INVALID_UNION_TYPE: &str = "Union types only allow nullable type (eg. `T | null`)";
const INVALID_MIXED_ENUM_MEMBER: &str =
    "Enum member type must be single type (eg. only `number` or `string`)";
const INVALID_REGISTRY_METHOD: &str = "Invalid TurboModuleRegistry method";

pub struct TurboModuleAnalyzer<'a> {
    pub diagnostics: Vec<OxcDiagnostic>,
    scoping: &'a Scoping,
    /// Symbol ID of `TurboModule` identifier's reference
    mod_type_sym_id: Option<SymbolId>,
    /// Symbol ID of `TurboModuleRegistry` identifier's reference
    mod_reg_sym_id: Option<SymbolId>,
    /// Symbol ID of `react-native` namespace's reference
    mod_ns_sym_id: Option<SymbolId>,
    /// TurboModule modules collected from the source code
    mods: BTreeMap<SymbolId, String>,
    /// Declarations collected from the source code
    decls: BTreeMap<SymbolId, TypeAnnotation>,
    /// TurboModule specs collected from the source code
    specs: BTreeMap<SymbolId, Spec>,
}

impl<'a> TurboModuleAnalyzer<'a> {
    fn new(scoping: &'a Scoping) -> Self {
        Self {
            scoping,
            diagnostics: vec![],
            mod_type_sym_id: None,
            mod_reg_sym_id: None,
            mod_ns_sym_id: None,
            specs: BTreeMap::new(),
            mods: BTreeMap::new(),
            decls: BTreeMap::new(),
        }
    }

    fn is_reg_call(&mut self, it: &CallExpression<'a>) -> bool {
        match &it.callee {
            Expression::StaticMemberExpression(member) => match &&member.object {
                Expression::Identifier(ident) => {
                    let sym_id = self.scoping.get_reference(ident.reference_id()).symbol_id();
                    let is_reg = self.mod_reg_sym_id == sym_id;
                    let is_get = member.property.name == REGISTRY_GET
                        || member.property.name == REGISTRY_GET_ENFORCING;

                    return if is_get {
                        is_reg
                    } else {
                        self.collect_error(INVALID_REGISTRY_METHOD, member.property.span);
                        false
                    };
                }
                Expression::StaticMemberExpression(outer_member) => {
                    let is_ns = if let Some(ref_id) = outer_member.object.get_identifier_reference()
                    {
                        let sym_id = self
                            .scoping
                            .get_reference(ref_id.reference_id())
                            .symbol_id();
                        self.mod_ns_sym_id == sym_id
                    } else {
                        false
                    };

                    if is_ns {
                        match &&member.object {
                            Expression::Identifier(ident) => {
                                let is_reg = ident.name == NATIVE_MODULE_REGISTRY;
                                let is_get = member.property.name == REGISTRY_GET
                                    || member.property.name == REGISTRY_GET_ENFORCING;

                                return if is_get {
                                    is_reg
                                } else {
                                    self.collect_error(
                                        INVALID_REGISTRY_METHOD,
                                        member.property.span,
                                    );
                                    false
                                };
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }

        false
    }

    fn as_spec_id(&mut self, it: &CallExpression<'a>) -> Option<SymbolId> {
        let spec_generic = match &it.type_arguments {
            Some(type_arguments) => match type_arguments.params.first() {
                Some(spec_generic) => {
                    // With generic argument, but not exactly one
                    // `TurboModuleRegistry.get<T, U, V>();`
                    if type_arguments.params.len() != 1 {
                        self.collect_error(
                            "TurboModule specification generic argument must be exactly one",
                            it.span,
                        );
                        return None;
                    }

                    spec_generic
                }
                None => {
                    // Without generic argument
                    // `TurboModuleRegistry.getEnforcing<>();`
                    self.collect_error(INVALID_NO_SPEC_GENERIC, it.span);
                    return None;
                }
            },
            None => {
                // Without generic argument
                // `TurboModuleRegistry.getEnforcing();`
                self.collect_error(INVALID_NO_SPEC_GENERIC, it.span);
                return None;
            }
        };

        if let TSType::TSTypeReference(type_ref) = spec_generic {
            match &type_ref.type_name {
                TSTypeName::IdentifierReference(ref_id) => {
                    let sym_id = self
                        .scoping
                        .get_reference(ref_id.reference_id())
                        .symbol_id();
                    sym_id
                }
                _ => {
                    self.collect_error("Invalid specification type reference", it.span);
                    return None;
                }
            }
        } else {
            self.collect_error(
                "Specification generic argument must be a type reference",
                it.span,
            );
            None
        }
    }

    fn as_mod_name(&mut self, it: &CallExpression<'a>) -> Option<String> {
        match it.arguments.first() {
            Some(Argument::StringLiteral(str_lit)) => {
                let mod_name = str_lit.value.as_str().to_string();

                if self.mods.values().find(|name| *name == &mod_name).is_some() {
                    self.diagnostics.push(
                        OxcDiagnostic::error("Duplicate module name").with_label(str_lit.span),
                    );
                    return None;
                }

                debug!("TurboModule found: {}", mod_name);
                Some(mod_name)
            }
            Some(_) => {
                self.collect_error("TurboModule name must be a string literal", it.span);
                return None;
            }
            None => {
                self.collect_error("TurboModule name is required", it.span);
                return None;
            }
        }
    }

    fn collect_mod(&mut self, it: &CallExpression<'a>) {
        if !self.is_reg_call(it) {
            return;
        }

        let spec_id = match self.as_spec_id(it) {
            Some(spec_id) => spec_id,
            None => return,
        };

        match self.as_mod_name(it) {
            Some(mod_name) => drop(self.mods.insert(spec_id, mod_name)),
            _ => {}
        };
    }

    fn collect_spec(&mut self, it: &TSInterfaceDeclaration<'a>) {
        let mut methods = vec![];

        for sig in &it.body.body {
            let as_method_res = match sig {
                TSSignature::TSMethodSignature(method_sig) => self.as_method(method_sig),
                TSSignature::TSPropertySignature(prop_sig) => self.as_method_from_prop(prop_sig),
                _ => return self.collect_error(INVALID_SPEC, it.span),
            };

            match as_method_res {
                Ok(method) => methods.push(method),
                Err(error) => return self.diagnostics.push(error),
            }
        }

        self.specs.insert(it.id.symbol_id(), Spec { methods });
    }

    fn collect_interface_type(&mut self, it: &TSInterfaceDeclaration<'a>) {
        if it.extends.len() > 0 {
            return self.collect_error(INVALID_SPEC, it.span);
        }

        // Collect type alias
        let mut props = vec![];
        for sig in &it.body.body {
            match sig {
                TSSignature::TSPropertySignature(prop_sig) => match self.as_prop(prop_sig) {
                    Ok(prop) => props.push(prop),
                    Err(error) => return self.diagnostics.push(error),
                },
                _ => return self.collect_error(INVALID_SPEC, it.span),
            }
        }

        let id = it.id.symbol_id();
        let name = it.id.name.to_string();

        self.decls.insert(
            id,
            TypeAnnotation::Object(ObjectTypeAnnotation { name, props }),
        );
    }

    fn collect_alias_type(&mut self, it: &TSTypeAliasDeclaration<'a>) {
        match &it.type_annotation {
            TSType::TSTypeLiteral(type_lit) => {
                let props = type_lit
                    .members
                    .iter()
                    .map(|member| match member {
                        TSSignature::TSPropertySignature(prop_sig) => self.as_prop(prop_sig),
                        _ => Err(error(INVALID_SPEC, type_lit.span)),
                    })
                    .collect::<Result<Vec<Prop>, OxcDiagnostic>>();

                match props {
                    Ok(props) => {
                        let id = it.id.symbol_id();
                        let name = it.id.name.to_string();

                        println!("!!! {:?} {:?} {:?}", id, name, props);

                        self.decls.insert(
                            id,
                            TypeAnnotation::Object(ObjectTypeAnnotation { name, props }),
                        );
                    }
                    Err(err) => return self.diagnostics.push(err),
                }
            }
            TSType::TSUnionType(union_type) => match self.as_nullable(&union_type) {
                Ok(type_annotation) => {
                    let id = it.id.symbol_id();
                    self.decls.insert(id, type_annotation);
                }
                Err(err) => return self.diagnostics.push(error(&err.to_string(), it.span)),
            },
            _ => self.collect_error(INVALID_SPEC, it.span),
        }
    }

    fn collect_enum_type(&mut self, it: &TSEnumDeclaration<'a>) {
        let mut members = vec![];
        let mut prev_num_raw_val = 0;
        let mut member_type = None;

        for (idx, member) in it.body.members.iter().enumerate() {
            match &member.initializer {
                Some(expr) => match expr {
                    Expression::NumericLiteral(num_lit) => {
                        if member_type.is_none() {
                            member_type = Some(TypeAnnotation::Number);
                        } else if !matches!(member_type.as_ref().unwrap(), TypeAnnotation::Number) {
                            return self.collect_error(INVALID_MIXED_ENUM_MEMBER, it.span);
                        }

                        prev_num_raw_val = num_lit.value as usize;
                        members.push(EnumMember {
                            name: member.id.static_name().to_string(),
                            value: EnumMemberValue::Number(num_lit.value),
                        });
                    }
                    Expression::StringLiteral(str_lit) => {
                        if member_type.is_none() {
                            member_type = Some(TypeAnnotation::String);
                        } else if !matches!(member_type.as_ref().unwrap(), TypeAnnotation::String) {
                            return self.collect_error(INVALID_MIXED_ENUM_MEMBER, it.span);
                        }

                        members.push(EnumMember {
                            name: member.id.static_name().to_string(),
                            value: EnumMemberValue::String(str_lit.value.into_string()),
                        });
                    }
                    _ => self.collect_error(INVALID_SPEC, it.span),
                },
                None => {
                    if member_type.is_none() {
                        member_type = Some(TypeAnnotation::Number);
                    } else if !matches!(member_type.as_ref().unwrap(), TypeAnnotation::Number) {
                        return self.collect_error(INVALID_MIXED_ENUM_MEMBER, it.span);
                    }

                    members.push(EnumMember {
                        name: member.id.static_name().to_string(),
                        value: EnumMemberValue::Number((prev_num_raw_val + idx) as f64),
                    });
                }
            };
        }

        self.decls.insert(
            it.id.symbol_id(),
            TypeAnnotation::Enum(EnumTypeAnnotation {
                name: it.id.name.to_string(),
                members,
            }),
        );
    }

    fn as_prop(&self, prop_sig: &TSPropertySignature<'a>) -> Result<Prop, OxcDiagnostic> {
        match &prop_sig.type_annotation {
            Some(type_annotation) => {
                let prop_name = match self.to_prop_name(&prop_sig.key) {
                    Ok(name) => name,
                    Err(e) => return Err(error(&e.to_string(), prop_sig.span)),
                };

                let type_annotation =
                    match self.as_type_annotation(&type_annotation.type_annotation) {
                        Ok(type_annotation) => type_annotation,
                        Err(e) => return Err(error(&e.to_string(), prop_sig.span)),
                    };

                Ok(Prop {
                    name: prop_name,
                    type_annotation,
                })
            }
            _ => Err(error(INVALID_SPEC, prop_sig.span)),
        }
    }

    fn as_method(&self, sig: &TSMethodSignature<'a>) -> Result<Method, OxcDiagnostic> {
        if sig.computed {
            return Err(error(INVALID_COMPUTED_SIG, sig.span));
        }

        if sig.optional {
            return Err(error(INVALID_OPTIONAL_SIG, sig.span));
        }

        let method_name = match &sig.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
            _ => return Err(error(INVALID_SPEC, sig.span)),
        };

        let params = sig
            .params
            .items
            .iter()
            .map(|param| {
                if param.decorators.len() > 0 {
                    return Err(error(INVALID_SPEC, param.span));
                }

                let param_name = param
                    .pattern
                    .kind
                    .get_identifier_name()
                    .ok_or_else(|| error(INVALID_SPEC, param.span))?;

                let param_type_annotation = param
                    .pattern
                    .type_annotation
                    .as_ref()
                    .ok_or_else(|| error(INVALID_SPEC, param.span))?;

                match self.as_type_annotation(&param_type_annotation.type_annotation) {
                    Ok(type_annotation) => Ok(Param {
                        name: param_name.to_string(),
                        type_annotation,
                    }),
                    Err(e) => Err(error(&e.to_string(), param.span)),
                }
            })
            .collect::<Result<Vec<Param>, OxcDiagnostic>>()?;

        let ret_type = sig
            .return_type
            .as_ref()
            .ok_or_else(|| error(INVALID_SPEC, sig.span))?;

        match self.as_type_annotation(&ret_type.type_annotation) {
            Ok(type_annotation) => Ok(Method {
                name: method_name,
                params,
                ret_type: type_annotation,
            }),
            Err(e) => Err(error(&e.to_string(), sig.span)),
        }
    }

    fn to_prop_name(&self, key: &PropertyKey) -> Result<String, anyhow::Error> {
        match key {
            PropertyKey::StaticIdentifier(ident) => Ok(ident.name.to_string()),
            _ => anyhow::bail!(INVALID_SPEC),
        }
    }

    fn as_method_from_prop(&self, sig: &TSPropertySignature<'a>) -> Result<Method, OxcDiagnostic> {
        if sig.computed {
            return Err(error(INVALID_COMPUTED_SIG, sig.span));
        }

        if sig.optional {
            return Err(error(INVALID_OPTIONAL_SIG, sig.span));
        }

        let prop_name = match self.to_prop_name(&sig.key) {
            Ok(name) => name,
            Err(e) => return Err(error(&e.to_string(), sig.span)),
        };

        let type_annotation = sig
            .type_annotation
            .as_ref()
            .ok_or_else(|| error(INVALID_SPEC, sig.span))?;

        match self.as_type_annotation(&type_annotation.type_annotation) {
            Ok(type_annotation) => Ok(Method {
                name: prop_name,
                params: vec![],
                ret_type: type_annotation,
            }),
            Err(e) => Err(error(&e.to_string(), sig.span)),
        }
    }

    fn as_type_annotation(&self, ts_type: &TSType<'a>) -> Result<TypeAnnotation, anyhow::Error> {
        match ts_type {
            TSType::TSVoidKeyword(..) => Ok(TypeAnnotation::Void),
            TSType::TSBooleanKeyword(..) => Ok(TypeAnnotation::Boolean),
            TSType::TSNumberKeyword(..) => Ok(TypeAnnotation::Number),
            TSType::TSStringKeyword(..) => Ok(TypeAnnotation::String),
            TSType::TSArrayType(arr_type) => {
                let type_annotation = self.as_type_annotation(&arr_type.element_type)?;
                Ok(TypeAnnotation::Array(Box::new(type_annotation)))
            }
            TSType::TSTypeReference(type_ref) => match &type_ref.type_name {
                TSTypeName::IdentifierReference(ref_id) => {
                    Ok(TypeAnnotation::Ref(ref_id.reference_id()))
                }
                _ => anyhow::bail!(INVALID_TYPE_REFERENCE),
            },
            TSType::TSUnionType(union_type) => self.as_nullable(&union_type),
            TSType::TSTypeLiteral { .. } => anyhow::bail!(INVALID_TYPE_LITERAL),
            TSType::TSFunctionType { .. } => anyhow::bail!(INVALID_FUNC_PARAM),
            _ => anyhow::bail!(INVALID_SPEC),
        }
    }

    fn as_nullable(&self, union_type: &TSUnionType<'a>) -> Result<TypeAnnotation, anyhow::Error> {
        if union_type.types.len() != 2 {
            anyhow::bail!(INVALID_UNION_TYPE);
        }

        let base = match (&union_type.types[0], &union_type.types[1]) {
            (TSType::TSNullKeyword(..), base) => base,
            (base, TSType::TSNullKeyword(..)) => base,
            _ => anyhow::bail!(INVALID_UNION_TYPE),
        };

        let base = match self.as_type_annotation(base)? {
            TypeAnnotation::Promise(..) => anyhow::bail!("Promise type cannot be nullable"),
            t @ _ => t,
        };

        Ok(TypeAnnotation::Nullable(Box::new(base)))
    }

    /// Check the specification interface extends `TurboModule` interface of 'react-native' package.
    fn is_spec(&self, it: &TSInterfaceDeclaration<'a>) -> bool {
        it.extends
            .iter()
            .find(|ex| {
                if let Some(ref_id) = ex.expression.get_identifier_reference() {
                    // Check if the expression is `TurboModule` of 'react-native' package
                    // eg. `import type { TurboModule } from 'react-native';`
                    let sym_id = self.scoping.get_reference(ref_id.reference_id()).symbol_id();
                    sym_id == self.mod_type_sym_id
                } else if let Some(member_expr) = ex.expression.get_member_expr() {
                    // Check if the expression is `Namespace.TurboModule` of 'react-native' package
                    // eg. `import * as Namespace from 'react-native'`
                    matches!(
                        member_expr.object(),
                        Expression::Identifier(ident)
                            if self.scoping.get_reference(ident.reference_id()).symbol_id() == self.mod_ns_sym_id
                            && member_expr.static_property_name() == Some(NATIVE_MODULE_INTERFACE)
                    )
                } else {
                    false
                }
            })
            .is_some()
    }

    /// Collect an error diagnostic
    fn collect_error(&mut self, message: &str, span: Span) {
        self.diagnostics
            .push(OxcDiagnostic::error(message.to_string()).with_label(span));
    }

    fn collect_types(
        type_annotation: &TypeAnnotation,
        scoping: &Scoping,
        decls: &BTreeMap<SymbolId, TypeAnnotation>,
        types: &mut Vec<ObjectTypeAnnotation>,
        enums: &mut Vec<EnumTypeAnnotation>,
    ) {
        match type_annotation {
            TypeAnnotation::Object(obj) => {
                types.push(obj.clone());

                obj.props.iter().for_each(|prop| {
                    TurboModuleAnalyzer::collect_types(
                        &prop.type_annotation,
                        scoping,
                        decls,
                        types,
                        enums,
                    )
                });
            }
            TypeAnnotation::Enum(enum_type) => enums.push(enum_type.clone()),
            _ => {}
        }
    }

    fn resolve_refs(
        type_annotation: &mut TypeAnnotation,
        scoping: &Scoping,
        decls: &BTreeMap<SymbolId, TypeAnnotation>,
    ) {
        match type_annotation {
            TypeAnnotation::Ref(ref_id) => {
                if let Some(sym_id) = scoping.get_reference(*ref_id).symbol_id() {
                    match decls.get(&sym_id) {
                        Some(resolved) => {
                            let mut resolved = resolved.clone();
                            TurboModuleAnalyzer::resolve_refs(&mut resolved, scoping, decls);
                            *type_annotation = resolved;
                        }
                        _ => {}
                    };
                }
            }
            TypeAnnotation::Object(obj) => {
                for prop in &mut obj.props {
                    TurboModuleAnalyzer::resolve_refs(&mut prop.type_annotation, scoping, decls);
                }
            }
            TypeAnnotation::Nullable(base_type) => {
                TurboModuleAnalyzer::resolve_refs(base_type, scoping, decls);
            }
            _ => {}
        }
    }

    fn into_schema(self) -> Result<Vec<Schema2>, anyhow::Error> {
        let mut schemas = Vec::with_capacity(self.specs.len());

        for (id, spec) in self.specs {
            let mut types = vec![];
            let mut enums = vec![];
            let module_name = self
                .mods
                .get(&id)
                .ok_or(anyhow::anyhow!("Module name not found"))?;

            let methods = spec
                .methods
                .into_iter()
                .map(|mut method| {
                    for param in &mut method.params {
                        TurboModuleAnalyzer::resolve_refs(
                            &mut param.type_annotation,
                            &self.scoping,
                            &self.decls,
                        );

                        TurboModuleAnalyzer::collect_types(
                            &param.type_annotation,
                            &self.scoping,
                            &self.decls,
                            &mut types,
                            &mut enums,
                        );
                    }

                    // Resolve type annotation of return value
                    TurboModuleAnalyzer::resolve_refs(
                        &mut method.ret_type,
                        &self.scoping,
                        &self.decls,
                    );

                    TurboModuleAnalyzer::collect_types(
                        &method.ret_type,
                        &self.scoping,
                        &self.decls,
                        &mut types,
                        &mut enums,
                    );

                    method
                })
                .collect::<Vec<Method>>();

            schemas.push(Schema2 {
                module_name: module_name.clone(),
                alias_map: types,
                enum_map: enums,
                methods,
            });
        }

        Ok(schemas)
    }
}

impl<'a> Visit<'a> for TurboModuleAnalyzer<'a> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.source.value.as_str() != REACT_NATIVE_PKG {
            return;
        }

        if let Some(specifiers) = &it.specifiers {
            for specifier in specifiers {
                self.visit_import_declaration_specifier(specifier);
            }
        }
    }

    fn visit_import_declaration_specifier(&mut self, it: &ImportDeclarationSpecifier<'a>) {
        match it {
            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                if let Some(symbol_id) = spec.local.symbol_id.get() {
                    let imported_name = match &spec.imported {
                        ModuleExportName::IdentifierName(ident) => ident.name,
                        ModuleExportName::IdentifierReference(ident) => ident.name,
                        ModuleExportName::StringLiteral(lit) => lit.value,
                    };

                    match imported_name.as_str() {
                        NATIVE_MODULE_INTERFACE => self.mod_type_sym_id = Some(symbol_id),
                        NATIVE_MODULE_REGISTRY => self.mod_reg_sym_id = Some(symbol_id),
                        _ => {}
                    };
                }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                self.mod_ns_sym_id = Some(spec.local.symbol_id());
            }
            _ => {}
        }
    }

    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        if it.declare {
            return;
        }

        if self.is_spec(it) {
            // Collect TurboModule spec
            self.collect_spec(it);
        } else {
            // Collect user defined type (interface)
            self.collect_interface_type(it);
        }
    }

    fn visit_ts_type_alias_declaration(&mut self, it: &TSTypeAliasDeclaration<'a>) {
        if it.declare {
            return;
        }

        // Collect user defined type (type alias)
        self.collect_alias_type(it);
    }

    fn visit_ts_enum_declaration(&mut self, it: &TSEnumDeclaration<'a>) {
        if it.declare {
            return;
        }

        // Collect user defined enum type
        self.collect_enum_type(it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // Collect TurboModule name from `TurboModuleRegistry.get()` or `TurboModuleRegistry.getEnforcing()`
        self.collect_mod(it);
    }
}

pub fn parse_schema(src: &str) -> Result<Vec<Schema2>, ParseError> {
    let allocator = Allocator::default();
    let source_type = SourceType::tsx();
    let ret = Parser::new(&allocator, src, source_type).parse();

    if ret.panicked || !ret.errors.is_empty() {
        return Err(ParseError::Oxc {
            diagnostics: ret.errors,
        });
    }

    let mut program = ret.program;
    let ret = SemanticBuilder::new().build(&program);

    if !ret.errors.is_empty() {
        return Err(ParseError::Oxc {
            diagnostics: ret.errors,
        });
    }

    let scoping = ret.semantic.into_scoping();
    let mut analyzer = TurboModuleAnalyzer::new(&scoping);

    analyzer.visit_program(&mut program);

    if analyzer.diagnostics.len() > 0 {
        return Err(ParseError::Oxc {
            diagnostics: analyzer.diagnostics,
        });
    }

    let schemas = analyzer.into_schema()?;

    Ok(schemas)
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::parser::turbo_module_analyzer::parse_schema;

    #[test]
    fn test_spec_interface() {
        let src = "
        import type { TurboModule } from 'react-native';

        export interface Spec extends TurboModule {
            myMethod(): void;
        }

        export default TurboModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let schemas = parse_schema(&src).unwrap();

        assert!(schemas.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_spec_interface_with_namespace() {
        let src = "
        import type * as ReactNative from 'react-native';

        export interface Spec extends ReactNative.TurboModule {
            myMethod(): void;
        }

        export default ReactNative.TurboModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let schemas = parse_schema(&src).unwrap();

        assert!(schemas.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_non_spec_interface() {
        let src = "
        import type { Unknown } from 'react-native';

        export interface Spec extends Unknown {
            myMethod(): void;
        }

        export default TurboModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = parse_schema(&src);

        assert!(result.is_err());
    }
}
