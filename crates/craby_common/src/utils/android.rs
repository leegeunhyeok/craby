use std::{fs, path::PathBuf};

pub fn is_gradle_configured(project_root: &PathBuf) -> Result<bool, anyhow::Error> {
    let gradle_path = build_gradle_path(project_root);

    fs::exists(&gradle_path)?;

    let mut passed = true;
    let content = fs::read_to_string(gradle_path)?;
    passed &= content.contains("externalNativeBuild");
    passed &= content.contains("cmake");
    passed &= content.contains("CMakeLists.txt");
    Ok(passed)
}

pub fn build_gradle_path(project_root: &PathBuf) -> PathBuf {
    project_root.join("android").join("build.gradle")
}
