use crate::utils::{
    collect_packages, is_valid_version, run_command, update_cargo_crate_versions,
    update_cargo_workspace_version, update_package_version,
};
use anyhow::Result;
use indoc::formatdoc;
use std::env;

pub fn run() -> Result<()> {
    let version = env::args()
        .nth(2)
        .ok_or_else(|| anyhow::anyhow!("Version is required"))?;

    if !is_valid_version(&version) {
        anyhow::bail!("Invalid version: {}", version);
    }

    println!("Updating version to {}", version);
    update_npm_package_version(&version)?;
    update_cargo_workspace_version(&version)?;
    update_cargo_crate_versions(&version)?;

    println!(
        "{}",
        formatdoc!(
            r#"
            To publish, commit changes and push:
            
            git add -A
            git commit -m "chore: release v{}"
            "#,
            version
        )
    );

    Ok(())
}

fn update_npm_package_version(version: &str) -> Result<()> {
    let packages = collect_packages()?;
    for package_info in &packages {
        println!("Updating package version: {}", package_info.name);
        update_package_version(package_info, version)?;
    }

    println!("Building napi package for prebuilt JS bundles...");
    run_command("yarn", &["workspace", "@craby/cli-bindings", "build"], None)?;
    Ok(())
}
