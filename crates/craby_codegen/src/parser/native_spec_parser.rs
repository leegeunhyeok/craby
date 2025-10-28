use log::debug;
use oxc::{
    allocator::Allocator,
    ast::ast::*,
    ast_visit::Visit,
    diagnostics::OxcDiagnostic,
    parser::Parser,
    semantic::{Scoping, SemanticBuilder, SymbolId},
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    constants::specs::*,
    parser::{types::*, utils::error},
    types::Schema,
};

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
const INVALID_REGISTRY_METHOD: &str = "Invalid NativeModuleRegistry method";
const INVALID_RESERVED_ARG_NAME_ID: &str = "Reserved argument name `it_` is not allowed";
const INVALID_RESERVED_METHOD_NAME_ID: &str = "Reserved method name `emit` is not allowed";

pub struct NativeModuleAnalyzer<'a> {
    pub diagnostics: Vec<OxcDiagnostic>,
    scoping: &'a Scoping,
    /// Symbol ID of `NativeModule` identifier's reference
    mod_type_sym_id: Option<SymbolId>,
    /// Symbol ID of `Signal` identifier's reference
    mod_signal_sym_id: Option<SymbolId>,
    /// Symbol ID of `NativeModuleRegistry` identifier's reference
    mod_reg_sym_id: Option<SymbolId>,
    /// Symbol ID of `react-native` namespace's reference
    mod_ns_sym_id: Option<SymbolId>,
    /// NativeModules collected from the source code
    mods: FxHashMap<SymbolId, String>,
    /// Declarations collected from the source code
    decls: FxHashMap<SymbolId, TypeAnnotation>,
    /// NativeModule specs collected from the source code
    specs: FxHashMap<SymbolId, Spec>,
}

impl<'a> NativeModuleAnalyzer<'a> {
    fn new(scoping: &'a Scoping) -> Self {
        Self {
            scoping,
            diagnostics: vec![],
            mod_type_sym_id: None,
            mod_signal_sym_id: None,
            mod_reg_sym_id: None,
            mod_ns_sym_id: None,
            specs: FxHashMap::default(),
            mods: FxHashMap::default(),
            decls: FxHashMap::default(),
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

        if let Some(mod_name) = self.as_mod_name(it) {
            drop(self.mods.insert(spec_id, mod_name))
        };
    }

    fn collect_spec(&mut self, it: &TSInterfaceDeclaration<'a>) {
        let mut methods = vec![];
        let mut signals = vec![];

        for sig in &it.body.body {
            match sig {
                TSSignature::TSMethodSignature(method_sig) => {
                    match self.try_into_method(method_sig) {
                        Ok(method) => methods.push(method),
                        Err(e) => return self.diagnostics.push(e),
                    }
                }
                TSSignature::TSPropertySignature(prop_sig) => {
                    match self.try_into_signal(prop_sig) {
                        Ok(signal) => signals.push(signal),
                        Err(e) => return self.diagnostics.push(e),
                    }
                }
                _ => return self.collect_error(INVALID_SPEC, it.span),
            };
        }

        let name = it.id.name.to_string();
        self.specs.insert(
            it.id.symbol_id(),
            Spec {
                name,
                methods,
                signals,
            },
        );
    }

    fn collect_interface_type(&mut self, it: &TSInterfaceDeclaration<'a>) {
        if let Err(e) = self.try_assert_reserved_type(&it.id.name) {
            return self.collect_error(&e.to_string(), it.span);
        };

        if !it.extends.is_empty() {
            return self.collect_error(INVALID_SPEC, it.span);
        }

        let id = it.id.symbol_id();
        let name = it.id.name.to_string();

        // Collect type alias
        let mut props = vec![];
        for sig in &it.body.body {
            match sig {
                TSSignature::TSPropertySignature(prop_sig) => match self.try_into_prop(prop_sig) {
                    Ok(prop) => props.push(prop),
                    Err(e) => return self.diagnostics.push(e),
                },
                _ => return self.collect_error(INVALID_SPEC, it.span),
            }
        }

        self.decls.insert(
            id,
            TypeAnnotation::Object(ObjectTypeAnnotation { name, props }),
        );
    }

    fn collect_alias_type(&mut self, it: &TSTypeAliasDeclaration<'a>) {
        if let Err(e) = self.try_assert_reserved_type(&it.id.name) {
            return self.collect_error(&e.to_string(), it.span);
        };

        if let Some(params) = &it.type_parameters {
            if !params.params.is_empty() {
                return self.collect_error("Type parameters are not supported", it.span);
            }
        }

        let id = it.id.symbol_id();
        let name = it.id.name.to_string();

        match &it.type_annotation {
            TSType::TSTypeLiteral(type_lit) => {
                let props = type_lit
                    .members
                    .iter()
                    .map(|member| match member {
                        TSSignature::TSPropertySignature(prop_sig) => self.try_into_prop(prop_sig),
                        _ => Err(error(INVALID_SPEC, type_lit.span)),
                    })
                    .collect::<Result<Vec<Prop>, OxcDiagnostic>>();

                match props {
                    Ok(props) => {
                        self.decls.insert(
                            id,
                            TypeAnnotation::Object(ObjectTypeAnnotation { name, props }),
                        );
                    }
                    Err(e) => self.diagnostics.push(e),
                }
            }
            TSType::TSUnionType(union_type) => match self.try_into_nullable(union_type) {
                Ok(type_annotation) => drop(self.decls.insert(id, type_annotation)),
                Err(e) => self.diagnostics.push(error(&e.to_string(), it.span)),
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
                        if let Some(type_annotation) = &member_type {
                            if !matches!(type_annotation, TypeAnnotation::Number) {
                                return self.collect_error(INVALID_MIXED_ENUM_MEMBER, it.span);
                            }
                        } else {
                            member_type = Some(TypeAnnotation::Number);
                        }

                        let raw = num_lit.value as usize;
                        prev_num_raw_val = raw;
                        let is_float = num_lit.raw_str().contains(".");

                        if is_float {
                            self.collect_error("Float number is not supported in enum", it.span);
                        } else {
                            members.push(EnumMember {
                                name: member.id.static_name().to_string(),
                                value: EnumMemberValue::Number(raw),
                            });
                        }
                    }
                    Expression::StringLiteral(str_lit) => {
                        if let Some(type_annotation) = &member_type {
                            if !matches!(type_annotation, TypeAnnotation::String) {
                                return self.collect_error(INVALID_MIXED_ENUM_MEMBER, it.span);
                            }
                        } else {
                            member_type = Some(TypeAnnotation::String);
                        }

                        members.push(EnumMember {
                            name: member.id.static_name().to_string(),
                            value: EnumMemberValue::String(str_lit.value.into_string()),
                        });
                    }
                    _ => self.collect_error(INVALID_SPEC, it.span),
                },
                None => {
                    if let Some(type_annotation) = &member_type {
                        if !matches!(type_annotation, TypeAnnotation::Number) {
                            return self.collect_error(INVALID_MIXED_ENUM_MEMBER, it.span);
                        }
                    } else {
                        member_type = Some(TypeAnnotation::Number);
                    }

                    members.push(EnumMember {
                        name: member.id.static_name().to_string(),
                        value: EnumMemberValue::Number(prev_num_raw_val + idx),
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

    fn as_spec_id(&mut self, it: &CallExpression<'a>) -> Option<SymbolId> {
        let spec_generic = match &it.type_arguments {
            Some(type_arguments) => match type_arguments.params.first() {
                Some(spec_generic) => {
                    // With generic argument, but not exactly one
                    // `NativeModuleRegistry.get<T, U, V>();`
                    if type_arguments.params.len() != 1 {
                        self.collect_error(
                            "NativeModule specification generic argument must be exactly one",
                            it.span,
                        );
                        return None;
                    }

                    spec_generic
                }
                None => {
                    // Without generic argument
                    // `NativeModuleRegistry.getEnforcing<>();`
                    self.collect_error(INVALID_NO_SPEC_GENERIC, it.span);
                    return None;
                }
            },
            None => {
                // Without generic argument
                // `NativeModuleRegistry.getEnforcing();`
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
                    None
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

                if self.mods.values().any(|name| name == &mod_name) {
                    self.diagnostics.push(
                        OxcDiagnostic::error("Duplicate module name").with_label(str_lit.span),
                    );
                    return None;
                }

                debug!("NativeModule found: {}", mod_name);
                Some(mod_name)
            }
            Some(_) => {
                self.collect_error("NativeModule name must be a string literal", it.span);
                None
            }
            None => {
                self.collect_error("NativeModule name is required", it.span);
                None
            }
        }
    }

    fn try_into_prop(&mut self, prop_sig: &TSPropertySignature<'a>) -> Result<Prop, OxcDiagnostic> {
        match &prop_sig.type_annotation {
            Some(type_annotation) => {
                let prop_name = match self.try_into_prop_name(&prop_sig.key) {
                    Ok(name) => name,
                    Err(e) => return Err(error(&e.to_string(), prop_sig.span)),
                };

                let type_annotation =
                    match self.try_into_type_annotation(&type_annotation.type_annotation) {
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

    fn try_into_method(&mut self, sig: &TSMethodSignature<'a>) -> Result<Method, OxcDiagnostic> {
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

        if method_name == RESERVED_METHOD_NAME_MODULE {
            return Err(error(INVALID_RESERVED_METHOD_NAME_ID, sig.span));
        }

        let params = sig
            .params
            .items
            .iter()
            .map(|param| {
                if !param.decorators.is_empty() {
                    return Err(error(INVALID_SPEC, param.span));
                }

                let param_name = param
                    .pattern
                    .kind
                    .get_identifier_name()
                    .ok_or_else(|| error(INVALID_SPEC, param.span))?;

                if param_name == RESERVED_ARG_NAME_MODULE {
                    return Err(error(INVALID_RESERVED_ARG_NAME_ID, param.span));
                }

                let param_type_annotation = param
                    .pattern
                    .type_annotation
                    .as_ref()
                    .ok_or_else(|| error(INVALID_SPEC, param.span))?;

                match self.try_into_type_annotation(&param_type_annotation.type_annotation) {
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

        match self.try_into_type_annotation(&ret_type.type_annotation) {
            Ok(type_annotation) => Ok(Method {
                name: method_name,
                params,
                ret_type: type_annotation,
            }),
            Err(e) => Err(error(&e.to_string(), sig.span)),
        }
    }

    fn try_into_signal(&mut self, sig: &TSPropertySignature<'a>) -> Result<Signal, OxcDiagnostic> {
        if sig.type_annotation.is_none() {
            return Err(error(INVALID_SPEC, sig.span));
        }

        let event_name = match &sig.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
            _ => return Err(error(INVALID_SPEC, sig.span)),
        };

        match &sig.type_annotation.as_ref().unwrap().type_annotation {
            TSType::TSTypeReference(type_ref) => match &type_ref.type_name {
                TSTypeName::IdentifierReference(ident_ref) => {
                    let sym_id = self
                        .scoping
                        .get_reference(ident_ref.reference_id())
                        .symbol_id();

                    if sym_id == self.mod_signal_sym_id {
                        Ok(Signal { name: event_name })
                    } else {
                        Err(error(INVALID_SPEC, sig.span))
                    }
                }
                _ => Err(error(INVALID_SPEC, sig.span)),
            },
            _ => Err(error(INVALID_SPEC, sig.span)),
        }
    }

    fn try_into_prop_name(&self, key: &PropertyKey) -> Result<String, anyhow::Error> {
        match key {
            PropertyKey::StaticIdentifier(ident) => Ok(ident.name.to_string()),
            _ => anyhow::bail!(INVALID_SPEC),
        }
    }

    fn try_into_type_annotation(
        &mut self,
        ts_type: &TSType<'a>,
    ) -> Result<TypeAnnotation, anyhow::Error> {
        match ts_type {
            TSType::TSVoidKeyword(..) => Ok(TypeAnnotation::Void),
            TSType::TSBooleanKeyword(..) => Ok(TypeAnnotation::Boolean),
            TSType::TSNumberKeyword(..) => Ok(TypeAnnotation::Number),
            TSType::TSStringKeyword(..) => Ok(TypeAnnotation::String),
            TSType::TSArrayType(arr_type) => {
                let type_annotation = self.try_into_type_annotation(&arr_type.element_type)?;
                Ok(TypeAnnotation::Array(Box::new(type_annotation)))
            }
            TSType::TSTypeReference(type_ref) => match &type_ref.type_name {
                TSTypeName::IdentifierReference(ident_ref) => {
                    if matches!(ident_ref.name.as_str(), RESERVED_TYPE_PROMISE) {
                        match &type_ref.type_arguments {
                            Some(type_args) if type_args.params.len() == 1 => {
                                let resolved_type = type_args.params.first().unwrap();
                                let resolved_type = self.try_into_type_annotation(resolved_type)?;
                                return Ok(TypeAnnotation::Promise(Box::new(resolved_type)));
                            }
                            _ => anyhow::bail!("Invalid promise type"),
                        }
                    }

                    Ok(TypeAnnotation::Ref(RefTypeAnnotation {
                        ref_id: ident_ref.reference_id(),
                        name: ident_ref.name.to_string(),
                    }))
                }
                _ => anyhow::bail!(INVALID_TYPE_REFERENCE),
            },
            TSType::TSUnionType(union_type) => self.try_into_nullable(union_type),
            TSType::TSTypeLiteral { .. } => anyhow::bail!(INVALID_TYPE_LITERAL),
            TSType::TSFunctionType { .. } => anyhow::bail!(INVALID_FUNC_PARAM),
            _ => anyhow::bail!(INVALID_SPEC),
        }
    }

    fn try_into_nullable(
        &mut self,
        union_type: &TSUnionType<'a>,
    ) -> Result<TypeAnnotation, anyhow::Error> {
        if union_type.types.len() != 2 {
            anyhow::bail!(INVALID_UNION_TYPE);
        }

        let base = match (&union_type.types[0], &union_type.types[1]) {
            (TSType::TSNullKeyword(..), base) => base,
            (base, TSType::TSNullKeyword(..)) => base,
            _ => anyhow::bail!(INVALID_UNION_TYPE),
        };

        let base = match self.try_into_type_annotation(base)? {
            TypeAnnotation::Promise(..) => anyhow::bail!("Promise type cannot be nullable"),
            base => base,
        };

        Ok(TypeAnnotation::Nullable(Box::new(base)))
    }

    /// Check the specification interface extends `NativeModule` interface of 'craby-modules' package.
    fn is_spec(&self, it: &TSInterfaceDeclaration<'a>) -> bool {
        it.extends.iter().any(|ex| {
            if let Some(ref_id) = ex.expression.get_identifier_reference() {
                // Check if the expression is `NativeModule` of 'craby-modules' package
                // eg. `import type { NativeModule } from 'craby-modules';`
                let sym_id = self
                    .scoping
                    .get_reference(ref_id.reference_id())
                    .symbol_id();
                sym_id == self.mod_type_sym_id
            } else if let Some(member_expr) = ex.expression.get_member_expr() {
                // Check if the expression is `Namespace.NativeModule` of 'craby-modules' package
                // eg. `import * as Namespace from 'craby-modules'`
                if let Expression::Identifier(ident) = member_expr.object() {
                    let sym_id = self.scoping.get_reference(ident.reference_id()).symbol_id();
                    member_expr.static_property_name() == Some(NATIVE_MODULE_INTERFACE)
                        && self
                            .mod_ns_sym_id
                            .zip(sym_id)
                            .is_some_and(|(id, s)| id == s)
                } else {
                    false
                }
            } else {
                false
            }
        })
    }

    fn is_reg_call(&mut self, it: &CallExpression<'a>) -> bool {
        if let Expression::StaticMemberExpression(member) = &it.callee {
            match &member.object {
                Expression::Identifier(ident) => {
                    let sym_id = self.scoping.get_reference(ident.reference_id()).symbol_id();
                    let is_reg = self
                        .mod_reg_sym_id
                        .zip(sym_id)
                        .is_some_and(|(id, s)| id == s);
                    let is_get = member.property.name == REGISTRY_GET
                        || member.property.name == REGISTRY_GET_ENFORCING;

                    return if is_get {
                        is_reg
                    } else {
                        self.collect_error(INVALID_REGISTRY_METHOD, member.property.span);
                        false
                    };
                }
                Expression::StaticMemberExpression(inner_member) => {
                    if let Some(ident) = inner_member.get_first_object().get_identifier_reference()
                    {
                        let sym_id = self.scoping.get_reference(ident.reference_id()).symbol_id();
                        let is_ns = self
                            .mod_ns_sym_id
                            .zip(sym_id)
                            .is_some_and(|(id, s)| id == s);

                        if !is_ns {
                            return false;
                        }
                    }

                    if let Expression::Identifier(_) = member.get_first_object() {
                        let name = member.property.name;
                        return name == REGISTRY_GET || name == REGISTRY_GET_ENFORCING;
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Collect an error diagnostic
    fn collect_error(&mut self, message: &str, span: Span) {
        self.diagnostics
            .push(OxcDiagnostic::error(message.to_string()).with_label(span));
    }

    fn collect_types(
        type_annotation: &TypeAnnotation,
        _scoping: &Scoping,
        _decls: &FxHashMap<SymbolId, TypeAnnotation>,
        types: &mut FxHashSet<TypeAnnotation>,
        enums: &mut FxHashSet<TypeAnnotation>,
    ) {
        match type_annotation {
            obj_type @ TypeAnnotation::Object(obj) => {
                types.insert(obj_type.clone());
                for prop in &obj.props {
                    NativeModuleAnalyzer::collect_types(
                        &prop.type_annotation,
                        _scoping,
                        _decls,
                        types,
                        enums,
                    );
                }
            }
            enum_type @ TypeAnnotation::Enum(..) => {
                enums.insert(enum_type.clone());
            }
            TypeAnnotation::Nullable(base_type) => {
                NativeModuleAnalyzer::collect_types(base_type, _scoping, _decls, types, enums);
            }
            TypeAnnotation::Promise(resolved_type) => {
                NativeModuleAnalyzer::collect_types(resolved_type, _scoping, _decls, types, enums);
            }
            _ => {}
        }
    }

    fn resolve_refs(
        type_annotation: &mut TypeAnnotation,
        scoping: &Scoping,
        decls: &FxHashMap<SymbolId, TypeAnnotation>,
    ) {
        match type_annotation {
            TypeAnnotation::Ref(RefTypeAnnotation { ref_id, .. }) => {
                match scoping.get_reference(*ref_id).symbol_id() {
                    Some(sym_id) => {
                        match decls.get(&sym_id) {
                            Some(resolved) => {
                                let mut resolved = resolved.clone();
                                NativeModuleAnalyzer::resolve_refs(&mut resolved, scoping, decls);
                                *type_annotation = resolved;
                            }
                            _ => unreachable!(
                                "Symbol not found (ref: {:?}, sym: {:?})",
                                ref_id, sym_id
                            ),
                        };
                    }
                    _ => unreachable!("Unknown type reference (ref: {:?})", ref_id),
                }
            }
            TypeAnnotation::Object(obj) => {
                for prop in &mut obj.props {
                    NativeModuleAnalyzer::resolve_refs(&mut prop.type_annotation, scoping, decls);
                }
            }
            TypeAnnotation::Nullable(base_type) => {
                NativeModuleAnalyzer::resolve_refs(base_type, scoping, decls);
            }
            TypeAnnotation::Promise(t) => {
                NativeModuleAnalyzer::resolve_refs(&mut *t, scoping, decls);
            }
            _ => {}
        }
    }

    fn try_assert_reserved_type(&self, name: &Atom<'a>) -> Result<(), anyhow::Error> {
        if matches!(name.as_str(), RESERVED_TYPE_PROMISE) {
            anyhow::bail!("Cannot use reserved type: {}", name.as_str());
        }

        if name.starts_with("Nullable") {
            anyhow::bail!("Nullable prefix is not allowed: {}", name.as_str());
        }

        Ok(())
    }

    fn try_into_schema(self) -> Result<Vec<Schema>, anyhow::Error> {
        let mut schemas = Vec::with_capacity(self.specs.len());

        for (id, mut spec) in self.specs {
            let mut types = FxHashSet::default();
            let mut enums = FxHashSet::default();
            let module_name = self
                .mods
                .get(&id)
                .ok_or(anyhow::anyhow!("NativeModule name not found"))?;

            let mut methods = spec
                .methods
                .into_iter()
                .map(|mut method| {
                    for param in &mut method.params {
                        NativeModuleAnalyzer::resolve_refs(
                            &mut param.type_annotation,
                            self.scoping,
                            &self.decls,
                        );

                        NativeModuleAnalyzer::collect_types(
                            &param.type_annotation,
                            self.scoping,
                            &self.decls,
                            &mut types,
                            &mut enums,
                        );
                    }

                    // Resolve type annotation of return value
                    NativeModuleAnalyzer::resolve_refs(
                        &mut method.ret_type,
                        self.scoping,
                        &self.decls,
                    );

                    NativeModuleAnalyzer::collect_types(
                        &method.ret_type,
                        self.scoping,
                        &self.decls,
                        &mut types,
                        &mut enums,
                    );

                    method
                })
                .collect::<Vec<Method>>();

            let mut aliases = types.into_iter().collect::<Vec<_>>();
            let mut enums = enums.into_iter().collect::<Vec<_>>();

            // Sort collected metadata to ensure deterministic output (for hash)
            aliases.sort_by_key(|v| v.as_object().unwrap().name.to_lowercase());
            enums.sort_by_key(|v| v.as_enum().unwrap().name.to_lowercase());
            methods.sort_by_key(|v| v.name.to_lowercase());
            spec.signals.sort_by_key(|v| v.name.to_lowercase());

            schemas.push(Schema {
                module_name: module_name.to_owned(),
                aliases,
                enums,
                methods,
                signals: spec.signals,
            });
        }

        Ok(schemas)
    }
}

impl<'a> Visit<'a> for NativeModuleAnalyzer<'a> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.source.value.as_str() != NATIVE_MODULE_PKG {
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
                        SIGNAL_TYPE => self.mod_signal_sym_id = Some(symbol_id),
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
            // Collect module spec
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
        // Collect module name from `NativeModuleRegistry.get()` or `NativeModuleRegistry.getEnforcing()`
        self.collect_mod(it);
    }
}

pub fn try_parse_schema(src: &str) -> Result<Vec<Schema>, ParseError> {
    let allocator = Allocator::default();
    let source_type = SourceType::tsx();
    let ret = Parser::new(&allocator, src, source_type).parse();

    if ret.panicked || !ret.errors.is_empty() {
        return Err(ParseError::Oxc {
            diagnostics: ret.errors,
        });
    }

    let program = ret.program;
    let ret = SemanticBuilder::new().build(&program);

    if !ret.errors.is_empty() {
        return Err(ParseError::Oxc {
            diagnostics: ret.errors,
        });
    }

    let scoping = ret.semantic.into_scoping();
    let mut analyzer = NativeModuleAnalyzer::new(&scoping);

    analyzer.visit_program(&program);

    if !analyzer.diagnostics.is_empty() {
        return Err(ParseError::Oxc {
            diagnostics: analyzer.diagnostics,
        });
    }

    debug!("Collected decls: {:?}", analyzer.decls);

    let schemas = analyzer.try_into_schema()?;

    Ok(schemas)
}

#[cfg(test)]
mod tests {
    use insta::{assert_debug_snapshot, assert_snapshot};

    use crate::{parser::native_spec_parser::try_parse_schema, types::Schema};

    #[test]
    fn test_common_spec() {
        let src = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface TestObject {
            foo: string;
            bar: number;
            baz: boolean;
            sub: SubObject | null;
        }

        export type SubObject = {
            a: string | null;
            b: number;
            c: boolean;
        };

        export type MaybeNumber = number | null;

        export enum MyEnum {
            Foo = 'foo',
            Bar = 'bar',
            Baz = 'baz',
        }

        export enum SwitchState {
            Off = 0,
            On = 1,
        }

        export interface Spec extends NativeModule {
            numericMethod(arg: number): number;
            booleanMethod(arg: boolean): boolean;
            stringMethod(arg: string): string;
            objectMethod(arg: TestObject): TestObject;
            arrayMethod(arg: number[]): number[];
            enumMethod(arg0: MyEnum, arg1: SwitchState): string;
            nullableMethod(arg: number | null): MaybeNumber;
            promiseMethod(arg: number): Promise<number>;
            onSignal: Signal;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('CrabyTest');

        ";
        let result = try_parse_schema(src).unwrap();

        assert!(result.len() == 1);
        assert_debug_snapshot!(result);
    }

    #[test]
    fn test_spec_interface() {
        let src = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let schemas = try_parse_schema(src).unwrap();

        assert!(schemas.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_spec_import_without_type() {
        let src = "
        import { NativeModuleRegistry, NativeModule, Signal } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let schemas = try_parse_schema(src).unwrap();

        assert!(schemas.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_spec_import_as_namespace() {
        let src = "
        import * as CrabyNativeModules from 'craby-modules';

        export interface Spec extends CrabyNativeModules.NativeModule {
            myMethod(): void;
        }

        export default CrabyNativeModules.NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let schemas = try_parse_schema(src).unwrap();

        assert!(schemas.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_spec_import_as_namespace_type() {
        let src = "
        import type * as CrabyNativeModules from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends CrabyNativeModules.NativeModule {
            myMethod(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let schemas = try_parse_schema(src).unwrap();

        assert!(schemas.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_signals() {
        let src = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            onFoo: Signal;
        }

        export const Foo = NativeModuleRegistry.getEnforcing<Spec>('TestModule');
        ";
        let schemas = try_parse_schema(src).unwrap();

        assert!(schemas.len() == 1);
        assert!(schemas[0].signals.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_ref_type() {
        let src = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        interface Foo {
            bar: string;
        }

        export interface Spec extends NativeModule {
            getFoo(): Promise<Foo>;
        }

        export const Foo = NativeModuleRegistry.getEnforcing<Spec>('TestModule');
        ";
        let schemas = try_parse_schema(src).unwrap();

        assert!(schemas.len() == 1);
        assert!(schemas[0].aliases.len() == 1);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_multiple_specs() {
        let src = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        type Common = { value: number };

        export interface Spec1 extends NativeModule {
            foo(arg: Common): void;
        }

        export interface Spec2 extends NativeModule {
            bar(arg: Common): void;
        }

        export const Foo = NativeModuleRegistry.getEnforcing<Spec1>('FooModule');
        export const Bar = NativeModuleRegistry.getEnforcing<Spec2>('BarModule');
        ";
        let schemas = try_parse_schema(src).unwrap();

        assert!(schemas.len() == 2);
        assert_debug_snapshot!(schemas);
    }

    #[test]
    fn test_non_spec_1() {
        let src = "
        import type { Unknown } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends Unknown {
            myMethod(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_non_spec_2() {
        let src = "
        import { NativeModuleRegistry } from 'react-native';

        export interface Spec {
            myMethod(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_spec_generic_1() {
        let src = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Unknown>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_spec_generic_2() {
        let src = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec, any>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_non_registry() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { Something } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(): void;
        }

        export default Something.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_non_registry_call() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(): void;
        }

        export default NativeModuleRegistry.foo<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_spec() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(): void;
        }

        export const Foo = NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        export const Bar = NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_enum_1() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        enum MyEnum {
            Foo = 'foo',
            Bar = 1
        }

        export interface Spec extends NativeModule {
            myMethod(arg: MyEnum): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_enum_2() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        enum MyEnum {
            Foo = 1,
            Bar = 3.14
        }

        export interface Spec extends NativeModule {
            myMethod(arg: MyEnum): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_reserved_type() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        type Promise = number;

        export interface Spec extends NativeModule {
            myMethod(arg: Promise): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_reserved_arg_name() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod(it_: number): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_reserved_method_name() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            emit(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_optional_method() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod?: () => void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_property_method() {
        let src: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            myMethod: () => void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let result = try_parse_schema(src);

        assert!(result.is_err());
    }

    #[test]
    fn test_hash() {
        let src_1: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        interface SomeObject {
            a: string;
            b: number;
            c: boolean;
        }

        export interface Spec extends NativeModule {
            foo(arg: SomeObject): SomeObject;
            bar(): void;
            baz(): void;
            onSignal: Signal;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";

        let src_2: &'static str = "
        import type { NativeModule, Signal } from 'craby-modules';
        import { NativeModuleRegistry } from 'craby-modules';

        export interface Spec extends NativeModule {
            foo(): void;
            bar(): void;
            baz(): void;
        }

        export default NativeModuleRegistry.getEnforcing<Spec>('MyModule');
        ";
        let schemas = try_parse_schema(src_1).unwrap();
        let hash_1 = Schema::to_hash(&schemas);

        let schemas = try_parse_schema(src_1).unwrap();
        let hash_2 = Schema::to_hash(&schemas);

        let schemas = try_parse_schema(src_2).unwrap();
        let hash_3 = Schema::to_hash(&schemas);

        assert_eq!(hash_1, hash_2);
        assert_ne!(hash_1, hash_3);
        assert_snapshot!([hash_1, hash_2, hash_3].join("\n"));
    }
}
