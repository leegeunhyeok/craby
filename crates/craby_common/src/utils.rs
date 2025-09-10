use convert_case::{Case, Casing};
use regex::Regex;

use crate::constants;

#[derive(Debug, Clone)]
pub struct SanitizedString(pub String);
impl SanitizedString {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

pub fn sanitize_str(value: &str) -> SanitizedString {
    let re = Regex::new(r"[^a-zA-Z]").unwrap();
    let str = re.replace_all(&value, "_").to_case(Case::Snake).to_string();
    SanitizedString(str)
}

pub fn pascal_case(value: &str) -> String {
    stringcase::pascal_case(value)
}

pub fn to_lib_name(str: &SanitizedString) -> String {
    format!("lib{}.a", str.0.replace("_", ""))
}

pub fn to_header_name(str: &SanitizedString) -> String {
    format!("lib{}.h", str.0.replace("_", ""))
}

pub fn to_impl_mod_name(str: &SanitizedString) -> String {
    format!("{}_{}", str.0, constants::IMPL_MOD_SUFFIX)
}

pub mod path {
    use std::path::PathBuf;

    use crate::constants::TEMP_DIR;

    pub fn tmp_dir(project_root: &PathBuf) -> PathBuf {
        project_root.join(TEMP_DIR)
    }

    pub fn crate_target_dir(project_root: &PathBuf, target: &String) -> PathBuf {
        project_root.join("target").join(target).join("release")
    }
}

pub mod android {
    use std::{fs, path::PathBuf};

    pub fn is_gradle_configured(project_root: &PathBuf) -> Result<bool, anyhow::Error> {
        let gradle_path = build_gradle_path(project_root);

        fs::exists(&gradle_path)?;

        let mut passed = true;
        let content = fs::read_to_string(gradle_path)?;
        passed &= content.contains("jniLibs.srcDirs");
        passed &= content.contains("src/main/jniLibs");
        Ok(passed)
    }

    pub fn build_gradle_path(project_root: &PathBuf) -> PathBuf {
        project_root.join("android").join("build.gradle")
    }
}

pub mod ios {
    use std::{fs, path::PathBuf};

    use regex::Regex;

    use super::SanitizedString;

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

    pub fn is_podspec_configured(project_root: &PathBuf) -> Result<bool, anyhow::Error> {
        let podspec_path = get_podspec_path(project_root)?;

        if podspec_path.is_none() {
            return Err(anyhow::anyhow!("`podspec` file not found"));
        }

        let mut passed = true;
        let podspec_path = podspec_path.unwrap();
        let content = fs::read_to_string(&podspec_path)?;
        passed &= content.contains(".vendored_frameworks");

        let re = Regex::new(r"ios/framework/lib\w+\.xcframework").unwrap();
        passed &= re.is_match(&content);

        Ok(passed)
    }

    pub fn xcframework_name(str: &SanitizedString) -> String {
        format!("lib{}.xcframework", str.0.replace("_", ""))
    }
}
