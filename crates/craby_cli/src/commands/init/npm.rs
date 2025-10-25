use serde::Deserialize;

const PKG_NAME: &str = "craby-modules";

#[derive(Deserialize, Debug)]
struct PackageInfo {
    version: String,
}

pub fn get_latest_version() -> anyhow::Result<String> {
    let url = format!("https://registry.npmjs.org/{}/latest", PKG_NAME);
    let response = reqwest::blocking::get(&url)?;
    let package_info: PackageInfo = response.json()?;

    Ok(package_info.version)
}
