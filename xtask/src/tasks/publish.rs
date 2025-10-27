use std::env;
use std::io::Write;
use std::path::PathBuf;

use crate::utils::{
    collect_packages, get_version_from_commit_message, is_main_ref, run_command,
    update_cargo_workspace_version, validate_package_versions, PackageInfo,
};
use anyhow::Result;

const CRABY_BINDINGS_PACKAGE_NAME: &str = "@craby/cli-bindings";

fn setup_npm() -> Result<()> {
    let npm_token = env::var("NPM_TOKEN").map_err(|_| anyhow::anyhow!("NPM_TOKEN is not set"))?;

    run_command(
        "yarn",
        &[
            "config",
            "set",
            "npmPublishRegistry",
            "https://registry.npmjs.org/",
        ],
        None,
    )?;

    run_command("yarn", &["config", "set", "npmAuthToken", &npm_token], None)?;

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

    run_command("npm", &["--version"], None)?;

    println!("Preparing NAPI package for npm publish...");
    run_command(
        "yarn",
        &["napi", "prepublish", "-t", "npm", "--no-gh-release"],
        Some(&napi_package.location),
    )?;

    println!("Publishing NAPI package to npm...");
    run_command(
        "yarn",
        &["npm", "publish", "--access", "public"],
        Some(&napi_package.location),
    )?;

    Ok(())
}

fn publish_packages(packages: &[PackageInfo]) -> Result<()> {
    let package_names = packages.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
    println!("Publishing packages: {:?}", package_names);

    run_command(
        "yarn",
        &[
            "workspaces",
            "foreach",
            "--all",
            "--no-private",
            "--exclude",
            CRABY_BINDINGS_PACKAGE_NAME,
            "exec",
            "yarn",
            "npm",
            "publish",
            "--access",
            "public",
        ],
        None,
    )?;

    Ok(())
}

fn publish_crates() -> Result<()> {
    env::var("CARGO_REGISTRY_TOKEN")
        .map_err(|_| anyhow::anyhow!("CARGO_REGISTRY_TOKEN is not set"))?;
    run_command("cargo", &["publish", "--workspace"], None)?;
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

    if !is_main_ref() {
        println!("Not a main branch, skipping publish");
        return Ok(());
    }

    let packages = collect_packages()?;
    validate_package_versions(&packages, &version)?;

    let napi_package = packages
        .iter()
        .find(|p| p.name == "@craby/cli-bindings")
        .ok_or_else(|| anyhow::anyhow!("NAPI package not found, unexpected error"))?;

    let general_packages = packages
        .iter()
        .filter(|p| p.name != "@craby/cli-bindings")
        .cloned()
        .collect::<Vec<_>>();

    // Crates
    update_cargo_workspace_version(&version)?;
    publish_crates()?;

    // NPM
    setup_npm()?;
    publish_napi_package(napi_package)?;
    publish_packages(&general_packages)?;

    println!("Publish complete");
    Ok(())
}
