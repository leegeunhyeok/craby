use std::{fs, path::Path};

use craby_common::utils::string::pascal_case;
use indoc::formatdoc;
use log::debug;

use crate::utils::terminal::run_command;

pub fn setup_react_native_project(dest_dir: &Path, pkg_name: &str) -> anyhow::Result<()> {
    let app_name = pascal_case(pkg_name);

    // Root package.json
    let root_package_json_path = dest_dir.join("package.json");
    let raw_package_json: String = fs::read_to_string(&root_package_json_path)?;
    let mut package_json = serde_json::from_str::<serde_json::Value>(&raw_package_json)?;
    if let Some(obj) = package_json.as_object_mut() {
        debug!("Inserting workspaces field");
        obj.insert("workspaces".to_string(), serde_json::json!(["example"]));

        fs::write(
            root_package_json_path,
            serde_json::to_string_pretty(&package_json)?,
        )?;
    }

    run_command(
        "npx",
        &[
            "@react-native-community/cli@latest",
            "init",
            app_name.as_str(),
            "--skip-install",
            "--skip-git-init",
        ],
        Some(&dest_dir.to_string_lossy()),
    )?;

    let react_native_dir = dest_dir.join(&app_name);
    let react_native_package_json_path = react_native_dir.join("package.json");
    let raw_package_json = fs::read_to_string(&react_native_package_json_path)?;
    let mut package_json = serde_json::from_str::<serde_json::Value>(&raw_package_json)?;
    if let Some(obj) = package_json.as_object_mut() {
        if let Some(dependencies) = obj.get_mut("dependencies") {
            if let Some(dependencies_obj) = dependencies.as_object_mut() {
                debug!("Inserting dependencies");
                dependencies_obj.insert(pkg_name.to_string(), serde_json::json!("workspace:*"));
            }
        }

        if let Some(dev_dependencies) = obj.get_mut("devDependencies") {
            if let Some(dev_dependencies_obj) = dev_dependencies.as_object_mut() {
                debug!("Inserting devDependencies");
                dev_dependencies_obj.insert("@craby/devkit".to_string(), serde_json::json!("*"));
            }
        }

        fs::write(
            react_native_package_json_path,
            serde_json::to_string_pretty(&package_json)?,
        )?;
    }

    let metro_config = formatdoc! {
        r#"
        const {{ getMetroConfig }} = require('@craby/devkit');
        const {{ getDefaultConfig, mergeConfig }} = require('@react-native/metro-config');

        /**
        * Metro configuration
        * https://reactnative.dev/docs/metro
        *
        * @type {{import('@react-native/metro-config').MetroConfig}}
        */
        const config = getMetroConfig(__dirname);

        module.exports = mergeConfig(getDefaultConfig(__dirname), config);
        "#
    };

    let react_native_config = formatdoc! {
        r#"
        const path = require('node:path');
        const {{ withWorkspaceModule }} = require('@craby/devkit');

        const modulePackagePath = path.resolve(__dirname, '..');
        const config = {{}};

        module.exports = withWorkspaceModule(config, modulePackagePath);
        "#
    };

    debug!("Overwriting config files");
    fs::write(react_native_dir.join("metro.config.js"), metro_config)?;
    fs::write(
        react_native_dir.join("react-native.config.js"),
        react_native_config,
    )?;

    if react_native_dir.try_exists()? {
        debug!(
            "Renaming React Native project to example: {:?}",
            react_native_dir
        );
        fs::rename(react_native_dir, dest_dir.join("example"))?;
    }

    Ok(())
}
