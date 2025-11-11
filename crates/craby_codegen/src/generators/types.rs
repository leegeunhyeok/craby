use std::path::PathBuf;

use crate::types::CodegenContext;

pub trait Template {
    type FileType;

    fn render(
        &self,
        ctx: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<TemplateResult>, anyhow::Error>;
}

pub trait Generator<T>
where
    T: Template,
{
    fn cleanup(ctx: &CodegenContext) -> Result<(), anyhow::Error>;
    fn generate(&self, ctx: &CodegenContext) -> Result<Vec<TemplateResult>, anyhow::Error>;
    fn template_ref(&self) -> &T;
}

pub trait GeneratorInvoker {
    fn invoke_generate(&self, ctx: &CodegenContext) -> Result<Vec<TemplateResult>, anyhow::Error>;
}

#[derive(Debug)]
pub struct TemplateResult {
    pub content: String,
    pub path: PathBuf,
    pub overwrite: bool,
}
