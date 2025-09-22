use std::path::PathBuf;

use crate::types::types::Project;

pub trait Template {
    type FileType;

    fn render(
        &self,
        project: &Project,
        file_type: &Self::FileType,
    ) -> Result<Vec<(PathBuf, String)>, anyhow::Error>;
}

pub trait Generator<T>
where
    T: Template,
{
    fn generate(&self, project: &Project) -> Result<Vec<GenerateResult>, anyhow::Error>;
    fn template_ref(&self) -> &T;
}

pub trait GeneratorInvoker {
    fn invoke_generate(&self, project: &Project) -> Result<Vec<GenerateResult>, anyhow::Error>;
}

#[derive(Debug)]
pub struct GenerateResult {
    pub content: String,
    pub path: PathBuf,
    pub overwrite: bool,
}
