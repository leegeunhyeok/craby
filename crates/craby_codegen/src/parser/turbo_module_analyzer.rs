use std::{collections::BTreeMap, path::PathBuf};

use log::{debug, error};
use oxc::{
    allocator::Allocator,
    ast::ast::*,
    ast_visit::{walk::walk_import_declaration_specifier, Visit},
    diagnostics::{DiagnosticService, GraphicalReportHandler, GraphicalTheme, OxcDiagnostic},
    parser::Parser,
    semantic::{ReferenceId, Scoping, SemanticBuilder, SymbolId},
};

const REACT_NATIVE_PKG: &str = "react-native";
const TURBO_INTERFACE: &str = "TurboModule";
const TURBO_REGISTRY: &str = "TurboModuleRegistry";

const INVALID_SPEC: &str = "Invalid specification";
const INVALID_TYPE_REFERENCE: &str = "Invalid type reference";
const INVALID_COMPUTED_SIG: &str = "Computed signature is not supported";
const INVALID_OPTIONAL_SIG: &str = "Optional signature is not supported";
const INVALID_NO_SPEC_GENERIC: &str = "TurboModule specification generic argument is required";
const INVALID_FUNC_PARAM: &str = "Function parameter is not supported";
const INVALID_TYPE_LITERAL: &str =
    "Type literal is not supported. Use defined type reference instead";

pub struct TurboModuleAnalyzer<'a> {
    pub diagnostics: Vec<OxcDiagnostic>,
    scoping: &'a Scoping,
    /// Symbol ID of `TurboModule` identifier's reference
    mod_type_sym_id: Option<SymbolId>,
    /// Symbol ID of `TurboModuleRegistry` identifier's reference
    mod_reg_sym_id: Option<SymbolId>,
    /// TurboModule modules collected from the source code
    mods: BTreeMap<String, SymbolId>,
    /// Declarations collected from the source code
    decls: BTreeMap<SymbolId, TypeAnnotation>,
    /// TurboModule specs collected from the source code
    specs: BTreeMap<SymbolId, Spec>,
}

pub struct Spec {
    /// Module methods
    pub methods: Vec<Method>,
}

pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: TypeAnnotation,
}

pub struct Param {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

pub struct Prop {
    pub name: String,
    pub type_annotation: TypeAnnotation,
}

pub struct EnumMember {
    pub name: String,
    pub value: EnumMemberValue,
}

pub enum EnumMemberValue {
    String(String),
    Number(f64),
}

pub enum TypeAnnotation {
    Void,
    Boolean,
    Number,
    String,
    Array(Box<TypeAnnotation>),
    Object(ObjectTypeAnnotation),
    Enum(EnumTypeAnnotation),
    Promise(Box<TypeAnnotation>),
    Nullable(Box<TypeAnnotation>),
    // Reference to `TypeAnnotation::Object` or `TypeAnnotation::Enum`
    Ref(ReferenceId),
}

pub struct ObjectTypeAnnotation {
    pub id: SymbolId,
    pub name: String,
    pub props: Vec<Prop>,
}

pub struct EnumTypeAnnotation {
    pub id: ReferenceId,
    pub name: String,
    pub members: Vec<EnumMember>,
}

impl<'a> TurboModuleAnalyzer<'a> {
    fn new(scoping: &'a Scoping) -> Self {
        Self {
            scoping,
            diagnostics: vec![],
            mod_type_sym_id: None,
            mod_reg_sym_id: None,
            specs: BTreeMap::new(),
            mods: BTreeMap::new(),
            decls: BTreeMap::new(),
        }
    }

    fn collect_mod(&mut self, it: &CallExpression<'a>) {
        match &it.callee {
            Expression::StaticMemberExpression(member) => match &&member.object {
                Expression::Identifier(ident) => {
                    let ref_id = ident.reference_id();
                    let sym_id = self.scoping.get_reference(ref_id).symbol_id();

                    if self.mod_reg_sym_id != sym_id {
                        return;
                    }

                    let spec_id = match &it.type_arguments {
                        Some(type_arguments) => match type_arguments.params.first() {
                            Some(spec_generic) => {
                                // With generic argument, but not exactly one
                                // `TurboModuleRegistry.get<T, U, V>();`
                                if type_arguments.params.len() != 1 {
                                    return self.collect_error(
                                        "TurboModule specification generic argument must be exactly one",
                                        it.span,
                                    );
                                }

                                if let TSType::TSTypeReference(type_ref) = spec_generic {
                                    let spec_id = match &type_ref.type_name {
                                        TSTypeName::IdentifierReference(id_ref) => {
                                            let ref_id = id_ref.reference_id();
                                            let sym_id = self
                                                .scoping
                                                .get_reference(ref_id)
                                                .symbol_id()
                                                .unwrap();

                                            sym_id
                                        }
                                        _ => {
                                            return self.collect_error(
                                                "Invalid specification type reference",
                                                it.span,
                                            )
                                        }
                                    };

                                    spec_id
                                } else {
                                    return self.collect_error(
                                        "Specification generic argument must be a type reference",
                                        it.span,
                                    );
                                }
                            }
                            None => {
                                // Without generic argument
                                // `TurboModuleRegistry.getEnforcing<>();`
                                return self.collect_error(INVALID_NO_SPEC_GENERIC, it.span);
                            }
                        },
                        None => {
                            // Without generic argument
                            // `TurboModuleRegistry.getEnforcing();`
                            return self.collect_error(INVALID_NO_SPEC_GENERIC, it.span);
                        }
                    };

                    debug!("TurboModule spec generic: {:#?}", spec_id);

                    let mod_name = member.property.name.as_str();
                    if !(mod_name == "get" || mod_name == "getEnforcing") {
                        return self
                            .collect_error("Invalid TurboModuleRegistry method", ident.span);
                    }

                    match it.arguments.first() {
                        Some(Argument::StringLiteral(str_lit)) => {
                            let mod_name = str_lit.value.as_str().to_string();

                            if self.mods.contains_key(&mod_name) {
                                self.diagnostics.push(
                                    OxcDiagnostic::error("Duplicate module name")
                                        .with_label(str_lit.span),
                                );
                                return;
                            }

                            debug!("TurboModule found: {}", mod_name);
                            self.mods.insert(mod_name, spec_id);
                        }
                        Some(_) => self
                            .collect_error("TurboModule name must be a string literal", ident.span),
                        None => self.collect_error("TurboModule name is required", ident.span),
                    }
                }
                _ => {}
            },

            _ => {}
        }
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

    fn collect_interface(&mut self, it: &TSInterfaceDeclaration<'a>) {
        if it.extends.len() > 0 {
            return self.collect_error(INVALID_SPEC, it.span);
        }

        // Collect type alias
        let mut props = vec![];

        for sig in &it.body.body {
            match sig {
                TSSignature::TSPropertySignature(prop_sig) => match &prop_sig.type_annotation {
                    Some(type_annotation) => {
                        let type_annotation =
                            self.as_type_annotation(&type_annotation.type_annotation);
                        let prop_name = self.to_prop_name(&prop_sig.key);

                        let prop_name = match prop_name {
                            Ok(name) => name,
                            Err(e) => return self.collect_error(&e.to_string(), prop_sig.span),
                        };

                        let type_annotation = match type_annotation {
                            Ok(type_annotation) => type_annotation,
                            Err(e) => return self.collect_error(&e.to_string(), prop_sig.span),
                        };

                        props.push(Prop {
                            name: prop_name,
                            type_annotation,
                        });
                    }
                    _ => return self.collect_error(INVALID_SPEC, prop_sig.span),
                },
                _ => return self.collect_error(INVALID_SPEC, it.span),
            }
        }

        let id = it.id.symbol_id();
        let name = it.id.name.to_string();
        self.decls.insert(
            id,
            TypeAnnotation::Object(ObjectTypeAnnotation { id, name, props }),
        );
    }

    fn as_method(&self, sig: &TSMethodSignature<'a>) -> Result<Method, OxcDiagnostic> {
        if sig.computed {
            return Err(self.error(INVALID_COMPUTED_SIG, sig.span));
        }

        if sig.optional {
            return Err(self.error(INVALID_OPTIONAL_SIG, sig.span));
        }

        let method_name = match &sig.key {
            PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
            _ => return Err(self.error(INVALID_SPEC, sig.span)),
        };

        let params = sig
            .params
            .items
            .iter()
            .map(|param| {
                if param.decorators.len() > 0 {
                    return Err(self.error(INVALID_SPEC, param.span));
                }

                let param_name = param
                    .pattern
                    .kind
                    .get_identifier_name()
                    .ok_or_else(|| self.error(INVALID_SPEC, param.span))?;

                let param_type_annotation = param
                    .pattern
                    .type_annotation
                    .as_ref()
                    .ok_or_else(|| self.error(INVALID_SPEC, param.span))?;

                match self.as_type_annotation(&param_type_annotation.type_annotation) {
                    Ok(type_annotation) => Ok(Param {
                        name: param_name.to_string(),
                        type_annotation,
                    }),
                    Err(error) => Err(self.error(&error.to_string(), param.span)),
                }
            })
            .collect::<Result<Vec<Param>, OxcDiagnostic>>()?;

        let ret_type = sig
            .return_type
            .as_ref()
            .ok_or_else(|| self.error(INVALID_SPEC, sig.span))?;

        match self.as_type_annotation(&ret_type.type_annotation) {
            Ok(type_annotation) => Ok(Method {
                name: method_name,
                params,
                ret_type: type_annotation,
            }),
            Err(error) => Err(self.error(&error.to_string(), sig.span)),
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
            return Err(self.error(INVALID_COMPUTED_SIG, sig.span));
        }

        if sig.optional {
            return Err(self.error(INVALID_OPTIONAL_SIG, sig.span));
        }

        let prop_name = match self.to_prop_name(&sig.key) {
            Ok(name) => name,
            Err(error) => return Err(self.error(&error.to_string(), sig.span)),
        };

        let type_annotation = sig
            .type_annotation
            .as_ref()
            .ok_or_else(|| self.error(INVALID_SPEC, sig.span))?;

        match self.as_type_annotation(&type_annotation.type_annotation) {
            Ok(type_annotation) => Ok(Method {
                name: prop_name,
                params: vec![],
                ret_type: type_annotation,
            }),
            Err(error) => Err(self.error(&error.to_string(), sig.span)),
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
                TSTypeName::IdentifierReference(id_ref) => {
                    Ok(TypeAnnotation::Ref(id_ref.reference_id()))
                }
                _ => anyhow::bail!(INVALID_TYPE_REFERENCE),
            },
            TSType::TSUnionType(union_type) => {
                if union_type.types.len() != 2 {
                    anyhow::bail!("Union types only allow nullable");
                }

                let base = match (&union_type.types[0], &union_type.types[1]) {
                    (TSType::TSNullKeyword(..), base) => base,
                    (base, TSType::TSNullKeyword(..)) => base,
                    _ => anyhow::bail!("Union types only allow nullable"),
                };

                Ok(TypeAnnotation::Nullable(Box::new(
                    self.as_type_annotation(base)?,
                )))
            }
            TSType::TSTypeLiteral { .. } => anyhow::bail!(INVALID_TYPE_LITERAL),
            TSType::TSFunctionType { .. } => anyhow::bail!(INVALID_FUNC_PARAM),
            _ => anyhow::bail!(INVALID_SPEC),
        }
    }

    /// Check the specification interface extends `TurboModule` interface of 'react-native' package.
    ///
    /// ```ts
    /// import type { TurboModule } from 'react-native';
    ///                                  // ^
    ///
    /// interface MySpec extends TurboModule {}
    ///                          // ^
    /// ```
    fn is_spec(&self, it: &TSInterfaceDeclaration<'a>) -> bool {
        it.extends
            .iter()
            .find(|ex| {
                if let Some(id_ref) = ex.expression.get_identifier_reference() {
                    let ref_id = id_ref.reference_id();
                    let sym_id = self.scoping.get_reference(ref_id).symbol_id();
                    self.mod_type_sym_id == sym_id
                } else {
                    false
                }
            })
            .is_some()
    }

    fn error(&self, message: &str, span: Span) -> OxcDiagnostic {
        OxcDiagnostic::error(message.to_string()).with_label(span)
    }

    /// Collect an error diagnostic
    fn collect_error(&mut self, message: &str, span: Span) {
        self.diagnostics
            .push(OxcDiagnostic::error(message.to_string()).with_label(span));
    }

    fn test(&self) {
        println!("Decls: {:#?}", self.decls.keys());
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
                        TURBO_INTERFACE => self.mod_type_sym_id = Some(symbol_id),
                        TURBO_REGISTRY => self.mod_reg_sym_id = Some(symbol_id),
                        _ => {}
                    };
                }
            }
            _ => {}
        }
    }

    fn visit_ts_enum_declaration(&mut self, it: &TSEnumDeclaration<'a>) {
        if it.declare {
            return;
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
            // Collect type alias (interface)
            self.collect_interface(it);
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.collect_mod(it);
    }
}

pub fn parse_schema(
    project_root: &PathBuf,
    path: &PathBuf,
    src: &str,
) -> Result<BTreeMap<String, Spec>, anyhow::Error> {
    let allocator = Allocator::default();
    let source_type = SourceType::tsx();
    let ret = Parser::new(&allocator, src, source_type).parse();

    if ret.panicked || !ret.errors.is_empty() {
        for error in &ret.errors {
            error!("{}", error);
        }
        return Err(anyhow::anyhow!("Parsing failed"));
    }

    let mut program = ret.program;
    let ret = SemanticBuilder::new().build(&program);

    if !ret.errors.is_empty() {
        anyhow::bail!("Semantic analysis failed");
    }

    let scoping = ret.semantic.into_scoping();
    let mut analyzer = TurboModuleAnalyzer::new(&scoping);
    analyzer.visit_program(&mut program);

    analyzer.test();

    if analyzer.diagnostics.len() > 0 {
        let handler = GraphicalReportHandler::new()
            .with_theme(GraphicalTheme::unicode())
            .with_links(false);

        let diagnostics =
            DiagnosticService::wrap_diagnostics(project_root, path, &src, analyzer.diagnostics);

        for diagnostic in diagnostics {
            let mut output = String::new();
            if handler
                .render_report(&mut output, diagnostic.as_ref())
                .is_ok()
            {
                eprint!("{}", output);
            }
        }

        anyhow::bail!("Failed to parse schema");
    } else {
        Ok(BTreeMap::new()) // TODO
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
