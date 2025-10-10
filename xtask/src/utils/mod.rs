use anyhow::Result;
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub location: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PackageJson {
    #[serde(flatten)]
    fields: IndexMap<String, serde_json::Value>,
}

pub fn run_command(command: &str, args: &[&str], cwd: Option<&str>) -> Result<()> {
    let mut cmd = Command::new(command);

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    match output.status.code() {
        Some(0) => Ok(()),
        _ => anyhow::bail!(
            "Command exited with code {}",
            output.status.code().unwrap_or(-1)
        ),
    }
}

pub fn is_valid_version(version: &str) -> bool {
    let re = regex::Regex::new(r"^[0-9]+\.[0-9]+\.[0-9]+").unwrap();
    re.is_match(version)
}

pub fn get_version_from_commit_message() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(&["log", "-1", "--pretty=%B"])
        .stdout(Stdio::piped())
        .output()?;

    if output.status.code() != Some(0) {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let version = String::from_utf8(output.stdout)?.trim().to_string();

    if is_valid_version(&version) {
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

pub fn is_main_ref() -> bool {
    match std::env::var("GITHUB_REF") {
        Ok(branch) => branch == "refs/heads/main",
        Err(_) => false,
    }
}

pub fn collect_packages() -> Result<Vec<PackageInfo>> {
    let output = Command::new("yarn")
        .args(&["workspaces", "list", "--json"])
        .stdout(Stdio::piped())
        .output()?;

    if output.status.code() != Some(0) {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let packages: Vec<PackageInfo> = stdout
        .lines()
        .filter(|line| line.contains("packages/"))
        .map(|line| serde_json::from_str(line))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(packages)
}

pub fn update_package_version(package_info: &PackageInfo, version: &str) -> Result<()> {
    let package_json_path = Path::new(&package_info.location).join("package.json");
    let raw_package_json = fs::read_to_string(&package_json_path)?;
    let mut package_json = serde_json::from_str::<PackageJson>(&raw_package_json)?;

    package_json.fields.insert(
        "version".to_string(),
        serde_json::Value::String(version.to_string()),
    );

    let updated_json = serde_json::to_string_pretty(&package_json)?;
    fs::write(&package_json_path, format!("{}\n", updated_json))?;

    Ok(())
}

pub fn validate_package_versions(package_infos: &[PackageInfo], version: &str) -> Result<()> {
    for package_info in package_infos {
        let package_json_path = Path::new(&package_info.location).join("package.json");
        let raw_package_json = fs::read_to_string(&package_json_path)?;
        let package_json = serde_json::from_str::<PackageJson>(&raw_package_json)?;

        let package_version = package_json
            .fields
            .get("version")
            .expect("Missing version in package.json")
            .as_str()
            .expect("Version is not a string");

        if package_version != version {
            anyhow::bail!(
                "Version mismatch for {}: {} !== {}",
                package_info.name,
                package_version,
                version
            );
        }
    }
    Ok(())
}
