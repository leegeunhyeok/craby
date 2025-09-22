use std::{collections::BTreeMap, path::PathBuf};

use log::{debug, error};
use oxc::{
    allocator::Allocator,
    ast::ast::*,
    ast_visit::Visit,
    diagnostics::{DiagnosticService, GraphicalReportHandler, GraphicalTheme, OxcDiagnostic},
    parser::Parser,
    semantic::{Scoping, SemanticBuilder, SymbolId},
};

const REACT_NATIVE_PKG: &str = "react-native";
const TURBO_INTERFACE: &str = "TurboModule";
const TURBO_REGISTRY: &str = "TurboModuleRegistry";

const INVALID_SPEC: &str = "Invalid specification";
const INVALID_COMPUTED_SIG: &str = "Computed signature is not supported";
const INVALID_OPTIONAL_SIG: &str = "Optional signature is not supported";

pub struct TurboModuleAnalyzer<'a> {
    pub diagnostics: Vec<OxcDiagnostic>,
    scoping: &'a Scoping,
    /// TurboModule specs collected from the source code
    mod_specs: BTreeMap<String, TurboModuleSpec>,
    /// Symbol ID of `TurboModule` identifier's reference
    mod_type_sym_id: Option<SymbolId>,
    /// Symbol ID of `TurboModuleRegistry` identifier's reference
    mod_reg_sym_id: Option<SymbolId>,
}

pub struct TurboModuleSpec {
    /// Native module name
    pub mod_name: String,
}

impl TurboModuleSpec {
    pub fn new(mod_name: &String) -> Self {
        Self {
            mod_name: mod_name.clone(),
        }
    }
}

impl<'a> TurboModuleAnalyzer<'a> {
    fn new(scoping: &'a Scoping) -> Self {
        Self {
            scoping,
            diagnostics: vec![],
            mod_specs: BTreeMap::new(),
            mod_type_sym_id: None,
            mod_reg_sym_id: None,
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

    /// Collect an error diagnostic
    fn collect_error(&mut self, message: &str, span: Span) {
        self.diagnostics
            .push(OxcDiagnostic::error(message.to_string()).with_label(span));
    }

    /// Into the collected TurboModule specs
    fn into_specs(self) -> BTreeMap<String, TurboModuleSpec> {
        self.mod_specs
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

    fn visit_ts_interface_declaration(&mut self, it: &TSInterfaceDeclaration<'a>) {
        if !self.is_spec(it) {
            return;
        }

        debug!("TurboModule spec found");

        for sig in &it.body.body {
            match sig {
                TSSignature::TSMethodSignature(method_sig) => {
                    if method_sig.optional {
                        return self.collect_error(INVALID_OPTIONAL_SIG, method_sig.span);
                    }

                    if method_sig.computed {
                        return self.collect_error(INVALID_COMPUTED_SIG, method_sig.span);
                    }

                    match &method_sig.key {
                        PropertyKey::Identifier(ident) => {
                            let ident_name = ident.name.as_str().to_string();

                            if ident_name == "getConstants" {
                                // TODO
                                return;
                            }
                        }
                        _ => {}
                    }
                }
                TSSignature::TSPropertySignature(prop_sig) => {
                    if prop_sig.optional {
                        return self.collect_error(INVALID_OPTIONAL_SIG, prop_sig.span);
                    }

                    if prop_sig.computed {
                        return self.collect_error(INVALID_COMPUTED_SIG, prop_sig.span);
                    }

                    match &prop_sig.type_annotation {
                        Some(type_annotation) => match &type_annotation.type_annotation {
                            TSType::TSFunctionType { .. } => {
                                println!("Function prop found!");
                            }
                            _ => self.collect_error(INVALID_SPEC, type_annotation.span),
                        },
                        None => self.collect_error(INVALID_SPEC, prop_sig.span),
                    }
                }
                _ => self.collect_error(INVALID_SPEC, it.span),
            }
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        println!("CallExpression: {:#?}", it);

        match &it.callee {
            Expression::StaticMemberExpression(member) => match &&member.object {
                Expression::Identifier(ident) => {
                    let ref_id = ident.reference_id();
                    let sym_id = self.scoping.get_reference(ref_id).symbol_id();

                    if self.mod_reg_sym_id != sym_id {
                        return;
                    }

                    // Without generic argument
                    // `TurboModuleRegistry.getEnforcing();`
                    if it.type_arguments.is_none() {
                        return self.collect_error(
                            "Module specification generic argument is required",
                            it.span,
                        );
                    }

                    let spec_generics = it.type_arguments.as_ref().unwrap();

                    // With generic argument, but not exactly one
                    // `TurboModuleRegistry.get<T, U, V>();`
                    if spec_generics.params.len() != 1 {
                        return self.collect_error(
                            "Module specification generic argument must be exactly one",
                            it.span,
                        );
                    }

                    let spec_generic = spec_generics.params.first();

                    let mod_name = member.property.name.as_str();
                    if !(mod_name == "get" || mod_name == "getEnforcing") {
                        return self
                            .collect_error("Invalid TurboModuleRegistry method", ident.span);
                    }

                    match it.arguments.first() {
                        Some(Argument::StringLiteral(str_lit)) => {
                            let mod_name = str_lit.value.as_str().to_string();

                            if self.mod_specs.contains_key(&mod_name) {
                                self.diagnostics.push(
                                    OxcDiagnostic::error("Duplicate module name")
                                        .with_label(str_lit.span),
                                );
                                return;
                            }

                            debug!("TurboModule found: {}", mod_name);
                            let spec = TurboModuleSpec::new(&mod_name);
                            self.mod_specs.insert(mod_name, spec);
                        }
                        _ => {}
                    }
                }
                _ => {}
            },

            _ => {}
        }
    }
}

pub fn parse_schema(
    project_root: &PathBuf,
    path: &PathBuf,
    src: &str,
) -> Result<BTreeMap<String, TurboModuleSpec>, anyhow::Error> {
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
    }

    Ok(analyzer.into_specs())
}

#[cfg(test)]
mod tests {
    // TODO
}
