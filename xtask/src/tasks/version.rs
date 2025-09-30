use crate::utils::{collect_packages, is_valid_version, update_package_version};
use anyhow::Result;
use std::env;

pub fn run() -> Result<()> {
    let version = env::args()
        .nth(2)
        .ok_or_else(|| anyhow::anyhow!("Version is required"))?;

    if !is_valid_version(&version) {
        anyhow::bail!("Invalid version: {}", version);
    }

    println!("Updating version to {}", version);

    let packages = collect_packages()?;
    for package_info in &packages {
        update_package_version(package_info, &version)?;
    }

    println!(
        r#"
To publish, commit changes and push:

git add -A
git commit -m "{}"
"#,
        version
    );

    Ok(())
}
