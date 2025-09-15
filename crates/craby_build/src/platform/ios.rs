use std::{fs, path::PathBuf};

use crate::{
    cargo::artifact::{ArtifactType, Artifacts},
    constants::{ios::Identifier, toolchain::Target},
};

use craby_codegen::constants::{objc_mod_provider_name, GENERATED_COMMENT};
use craby_common::{
    config::CompleteCrabyConfig, constants::lib_base_name, utils::string::SanitizedString,
};
use log::debug;

const IOS_TARGETS: [Target; 2] = [
    Target::Ios(Identifier::Arm64),
    Target::Ios(Identifier::Arm64Simulator),
];

pub fn crate_libs<'a>(config: &'a CompleteCrabyConfig) -> Result<(), anyhow::Error> {
    let ios_base_path = ios_base_path(&config.project_root);

    if ios_base_path.exists() {
        fs::remove_dir_all(&ios_base_path)?;
        debug!("Cleaned up existing iOS base directory");
    }

    let xcframework_path = create_xcframework(&config)?;

    for target in IOS_TARGETS {
        if let Target::Ios(identifier) = &target {
            let artifacts = Artifacts::get_artifacts(config, &target)?;
            let identifier = identifier.to_str();

            // ios/src
            artifacts.copy_to(ArtifactType::Src, &ios_base_path.join("src"))?;

            // ios/include
            artifacts.copy_to(ArtifactType::Header, &ios_base_path.join("include"))?;

            // ios/framework/lib{lib_name}.xcframework/{identifier}
            artifacts.copy_to(ArtifactType::Lib, &xcframework_path.join(identifier))?;
        } else {
            unreachable!();
        }
    }

    fs::write(
        ios_base_path.join(format!(
            "{}.mm",
            objc_mod_provider_name(&config.project.name)
        )),
        format!(
            "// {}\n{}\n",
            GENERATED_COMMENT,
            craby_codegen::platform::ios::template::objc_mod_provider(&config.project.name)
        ),
    )?;

    Ok(())
}

fn create_xcframework(config: &CompleteCrabyConfig) -> Result<PathBuf, anyhow::Error> {
    let name = SanitizedString::from(&config.project.name);
    let lib_base_name = lib_base_name(&name);
    let info_plist_content = craby_codegen::platform::ios::template::info_plist(
        &config.project.name,
        Identifier::Arm64.to_str(),
        Identifier::Arm64Simulator.to_str(),
    );
    let framework_path = ios_base_path(&config.project_root).join("framework");
    let xcframework_path =
        framework_path.join(format!("lib{}.xcframework", lib_base_name.to_string()));

    if xcframework_path.exists() {
        fs::remove_dir_all(&xcframework_path)?;
        debug!("Cleaned up existing iOS xcframework");
    }

    fs::create_dir_all(&xcframework_path)?;

    let info_plist_path = xcframework_path.join("Info.plist");
    fs::write(info_plist_path, info_plist_content)?;

    Ok(xcframework_path)
}

fn ios_base_path(project_root: &PathBuf) -> PathBuf {
    project_root.join("ios")
}
