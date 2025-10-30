use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn is_gradle_configured(project_root: &Path) -> Result<bool, anyhow::Error> {
    let gradle_path = build_gradle_path(project_root);

    let mut passed = true;
    let content = fs::read_to_string(gradle_path)?;
    passed &= content.contains("externalNativeBuild");
    passed &= content.contains("cmake");
    passed &= content.contains("CMakeLists.txt");
    Ok(passed)
}

pub fn is_valid_android_package_name(package_name: &str) -> Result<bool, anyhow::Error> {
    let re = regex::Regex::new(r"^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)*$")?;
    Ok(re.is_match(package_name))
}

pub fn build_gradle_path(project_root: &Path) -> PathBuf {
    project_root.join("android").join("build.gradle")
}
