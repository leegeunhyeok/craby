use std::{fs, path::Path};

use craby_common::utils::string::pascal_case;
use indoc::formatdoc;
use log::debug;

use crate::utils::{
    log::success,
    template::TemplateData,
    terminal::{run_command, with_spinner},
};

pub fn setup_react_native_project(
    dest_dir: &Path,
    pkg_name: &str,
    template_data: &TemplateData,
) -> anyhow::Result<()> {
    with_spinner("Setting up React Native project...", |_| {
        if let Err(e) = setup_react_native_project_impl(dest_dir, pkg_name, template_data) {
            anyhow::bail!("Failed to setup React Native project: {}", e);
        }
        Ok(())
    })?;
    success("React Native project setup completed");
    Ok(())
}

pub fn setup_react_native_project_impl(
    dest_dir: &Path,
    pkg_name: &str,
    template_data: &TemplateData,
) -> anyhow::Result<()> {
    let app_name = format!("{}Example", pascal_case(pkg_name));

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

    // <example>/package.json
    let project_dir = dest_dir.join(&app_name);
    let react_native_pkg_json_path = project_dir.join("package.json");
    let raw_pkg_json = fs::read_to_string(&react_native_pkg_json_path)?;
    let mut pkg_json = serde_json::from_str::<serde_json::Value>(&raw_pkg_json)?;
    if let Some(obj) = pkg_json.as_object_mut() {
        if let Some(dependencies) = obj.get_mut("dependencies") {
            if let Some(dependencies_obj) = dependencies.as_object_mut() {
                debug!("Inserting dependencies");
                dependencies_obj.insert(pkg_name.to_string(), serde_json::json!("workspace:*"));
            }
        }

        if let Some(dev_dependencies) = obj.get_mut("devDependencies") {
            if let Some(dev_dependencies_obj) = dev_dependencies.as_object_mut() {
                debug!("Inserting devDependencies");
                dev_dependencies_obj.insert(
                    "@craby/devkit".to_string(),
                    serde_json::json!(template_data["pkg_version"].clone()),
                );
            }
        }

        fs::write(
            react_native_pkg_json_path,
            serde_json::to_string_pretty(&pkg_json)?,
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

    debug!("Overwriting files");
    fs::write(project_dir.join("metro.config.js"), metro_config)?;
    fs::write(
        project_dir.join("react-native.config.js"),
        react_native_config,
    )?;

    let dest_dir = dest_dir.join("example");
    debug!(
        "Renaming React Native project {:?} to {:?}",
        project_dir, dest_dir
    );
    fs::rename(project_dir, dest_dir)?;

    Ok(())
}
