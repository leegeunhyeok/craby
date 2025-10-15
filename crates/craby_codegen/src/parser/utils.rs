use std::path::PathBuf;

use oxc::{
    diagnostics::{DiagnosticService, GraphicalReportHandler, GraphicalTheme, OxcDiagnostic},
    span::Span,
};

pub struct RenderReportOptions<'a> {
    pub project_root: &'a PathBuf,
    pub path: &'a PathBuf,
    pub src: &'a str,
}

pub fn render_report<'a>(diagnostics: Vec<OxcDiagnostic>, opts: RenderReportOptions<'a>) {
    let handler = GraphicalReportHandler::new()
        .with_theme(GraphicalTheme::unicode())
        .with_links(false);

    for diagnostic in
        DiagnosticService::wrap_diagnostics(opts.project_root, opts.path, opts.src, diagnostics)
    {
        let mut output = String::new();
        if handler
            .render_report(&mut output, diagnostic.as_ref())
            .is_ok()
        {
            eprint!("{}", output);
        }
    }
}

pub fn error(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error(message.to_string()).with_label(span)
}
