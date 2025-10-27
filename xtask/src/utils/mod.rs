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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub location: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PackageJson {
    #[serde(flatten)]
    fields: IndexMap<String, serde_json::Value>,
}

const VERSION_REGEX: &str = r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?";

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
    let re = regex::Regex::new(VERSION_REGEX).unwrap();
    re.is_match(version)
}

pub fn parse_version_from_commit_message(msg: &str) -> Option<String> {
    println!("Parsing version from commit message: {:#?}", msg);
    let re = regex::Regex::new(format!("chore: release v({})", VERSION_REGEX).as_str()).unwrap();
    let captures = re.captures(msg);

    if let Some(captures) = captures {
        let version = captures.get(1).unwrap().as_str().to_string();

        if is_valid_version(&version) {
            return Some(version);
        }
    }

    None
}

pub fn get_version_from_commit_message() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%B"])
        .stdout(Stdio::piped())
        .output()?;

    if output.status.code() != Some(0) {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let msg = String::from_utf8(output.stdout)?.trim().to_string();
    let version = parse_version_from_commit_message(&msg);
    println!("Parsed version: {:?}", version);

    Ok(version)
}

pub fn is_main_ref() -> bool {
    match std::env::var("GITHUB_REF") {
        Ok(branch) => branch == "refs/heads/main",
        Err(_) => false,
    }
}

pub fn collect_packages() -> Result<Vec<PackageInfo>> {
    let output = Command::new("yarn")
        .args(["workspaces", "list", "--json"])
        .stdout(Stdio::piped())
        .output()?;

    if output.status.code() != Some(0) {
        anyhow::bail!(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let packages: Vec<PackageInfo> = stdout
        .lines()
        .filter(|line| line.contains("packages/"))
        .map(serde_json::from_str)
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

pub fn collect_crates() -> Result<Vec<CrateInfo>> {
    let root_dir = std::env::current_dir()?;
    let crates = fs::read_dir("crates")?
        .filter_map(|entry| -> Option<CrateInfo> {
            let entry = entry.unwrap();
            let file_type = entry.file_type().unwrap();

            if file_type.is_dir() {
                Some(CrateInfo {
                    location: root_dir
                        .join("crates")
                        .join(entry.file_name())
                        .to_string_lossy()
                        .to_string(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(crates)
}

pub fn update_cargo_workspace_version(version: &str) -> Result<()> {
    println!("Updating cargo workspace version...");
    let cargo_toml_path = std::env::current_dir()?.join("Cargo.toml");
    let raw_cargo_toml = fs::read_to_string(&cargo_toml_path)?;
    let mut doc = raw_cargo_toml.parse::<toml_edit::DocumentMut>()?;
    doc["workspace"]["package"]["version"] = toml_edit::value(version);
    fs::write(&cargo_toml_path, doc.to_string())?;
    Ok(())
}

pub fn update_cargo_crate_versions(version: &str) -> Result<()> {
    let pattern =
        regex::Regex::new(r#"(craby[a-zA-Z0-9_]*\s*=\s*\{\s*version\s*=\s*")[^"]+(")"#).unwrap();
    for crate_info in collect_crates()? {
        println!("Updating cargo crate version: {}", crate_info.location);
        let cargo_toml_path = Path::new(&crate_info.location).join("Cargo.toml");
        let raw_cargo_toml = fs::read_to_string(&cargo_toml_path)?;
        let updated_cargo_toml = pattern
            .replace_all(&raw_cargo_toml, format!(r#"${{1}}{}${{2}}"#, version))
            .to_string();
        fs::write(&cargo_toml_path, updated_cargo_toml)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_version() {
        assert!(is_valid_version("1.2.3"));
        assert!(is_valid_version("1.0.0-alpha.1"));
        assert!(is_valid_version("123.123.123"));
        assert!(!is_valid_version("123"));
        assert!(!is_valid_version("1.0"));
    }

    #[test]
    fn test_parse_version_from_commit_message() {
        let r1 = parse_version_from_commit_message("chore: release v1.2.3").unwrap();
        let r2 = parse_version_from_commit_message("chore: release v1.0.0-alpha.1").unwrap();
        let r3 = parse_version_from_commit_message("chore: release v123.123.123").unwrap();
        let r4 = parse_version_from_commit_message("chore: release v123");
        let r5 = parse_version_from_commit_message("chore: release v1.0");
        let r6 = parse_version_from_commit_message("chore: release 1.0");

        assert_eq!(r1, "1.2.3");
        assert_eq!(r2, "1.0.0-alpha.1");
        assert_eq!(r3, "123.123.123");
        assert!(r4.is_none());
        assert!(r5.is_none());
        assert!(r6.is_none());
    }
}
