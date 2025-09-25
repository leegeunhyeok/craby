use std::path::PathBuf;

use crate::types::CodegenContext;

pub trait Template {
    type FileType;

    fn render(
        &self,
        ctx: &CodegenContext,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error>;
}

pub trait Generator<T>
where
    T: Template,
{
    fn generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error>;
    fn template_ref(&self) -> &T;
}

pub trait GeneratorInvoker {
    fn invoke_generate(&self, ctx: &CodegenContext) -> Result<Vec<GenerateResult>, anyhow::Error>;
}

#[derive(Debug)]
pub struct GenerateResult {
    pub content: String,
    pub path: PathBuf,
    pub overwrite: bool,
}
