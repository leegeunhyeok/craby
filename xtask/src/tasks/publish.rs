use crate::utils::{
    collect_packages, get_version_from_commit_message, validate_package_versions, PackageInfo,
};
use anyhow::Result;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn setup_npm() -> Result<()> {
    let npm_token = env::var("NPM_TOKEN").map_err(|_| anyhow::anyhow!("NPM_TOKEN is not set"))?;

    Command::new("yarn")
        .args(&[
            "config",
            "set",
            "npmPublishRegistry",
            "https://registry.npmjs.org/",
        ])
        .status()?;

    Command::new("yarn")
        .args(&["config", "set", "npmAuthToken", &npm_token])
        .status()?;

    let npmrc_content = format!("//registry.npmjs.org/:_authToken={}\n", npm_token);
    let home_dir = PathBuf::from(env::var("HOME")?);
    let npmrc_path = home_dir.join(".npmrc");

    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&npmrc_path)?
        .write_all(npmrc_content.as_bytes())?;

    Ok(())
}

fn publish_napi_package(napi_package: &PackageInfo) -> Result<()> {
    println!("Publishing NAPI package: {}", napi_package.name);

    Command::new("yarn")
        .args(&["napi", "prepublish", "-t", "npm", "--skip-gh-release"])
        .current_dir(&napi_package.location)
        .status()?;

    Command::new("yarn")
        .args(&["npm", "publish", "--access", "public"])
        .current_dir(&napi_package.location)
        .status()?;

    Ok(())
}

fn publish_packages(packages: &[PackageInfo]) -> Result<()> {
    for package_info in packages {
        println!("Publishing {}...", package_info.name);
        Command::new("yarn")
            .args(&[
                "workspace",
                &package_info.name,
                "npm",
                "publish",
                "--access",
                "public",
            ])
            .status()?;
    }
    Ok(())
}

pub fn run() -> Result<()> {
    let version = match get_version_from_commit_message()? {
        Some(v) => v,
        None => {
            println!("Not a release, skipping publish");
            return Ok(());
        }
    };

    let packages = collect_packages()?;
    validate_package_versions(&packages, &version)?;

    let napi_package = packages
        .iter()
        .find(|p| p.name == "@craby/cli-bindings")
        .ok_or_else(|| anyhow::anyhow!("NAPI package not found, unexpected error"))?;

    let general_packages: Vec<_> = packages
        .iter()
        .filter(|p| p.name != "@craby/cli-bindings")
        .cloned()
        .collect();

    setup_npm()?;
    publish_napi_package(napi_package)?;
    publish_packages(&general_packages)?;

    println!("Publish complete");
    Ok(())
}
