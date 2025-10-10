use std::{fs, path::PathBuf, process::Command};

use regex::Regex;

use super::string::SanitizedString;

pub fn get_podspec_path(project_root: &PathBuf) -> Result<Option<String>, anyhow::Error> {
    let files = fs::read_dir(project_root)?;

    for file in files {
        let file = file?;
        let file_name = file.file_name().to_string_lossy().to_string();

        if file_name.ends_with(".podspec") {
            return Ok(Some(file_name));
        }
    }

    Ok(None)
}

pub fn is_xcode_cli_tools_installed() -> Result<bool, anyhow::Error> {
    let res = Command::new("xcode-select").args(["--version"]).output()?;
    Ok(res.status.success())
}

pub fn is_podspec_configured(project_root: &PathBuf) -> Result<bool, anyhow::Error> {
    let mut passed = true;
    let podspec_path = get_podspec_path(project_root)?
        .ok_or_else(|| anyhow::anyhow!("`podspec` file not found"))?;
    let content = fs::read_to_string(&podspec_path)?;
    passed &= content.contains(".vendored_frameworks");

    let re = Regex::new(r"ios/framework/lib\w+\.xcframework").unwrap();
    passed &= re.is_match(&content);

    Ok(passed)
}

pub fn xcframework_name(str: &SanitizedString) -> String {
    format!("lib{}.xcframework", str.0.replace("_", ""))
}
