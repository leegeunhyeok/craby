use std::{fs, path::PathBuf};

use craby_common::{
    config::CompleteCrabyConfig,
    constants::{crate_target_dir, cxx_bridge_dir, cxx_bridge_include_dir, lib_base_name},
    utils::{fs::collect_files, string::SanitizedString},
};
use log::debug;

use crate::constants::toolchain::Target;

#[derive(Clone)]
pub struct Artifacts {
    pub identifier: String,
    pub srcs: Vec<PathBuf>,
    pub headers: Vec<PathBuf>,
    pub libs: Vec<PathBuf>,
}

#[derive(PartialEq)]
pub enum ArtifactType {
    Src,
    Header,
    Lib,
}

const CXX_SRC_EXTS: &[&str] = &["c", "cc"];
const CXX_HEADER_EXTS: &[&str] = &["h", "hh"];

impl Artifacts {
    pub fn get_artifacts(
        config: &CompleteCrabyConfig,
        target: &Target,
    ) -> Result<Artifacts, anyhow::Error> {
        let cxx_bridge_dir = cxx_bridge_dir(&config.project_root, target.to_str());
        let cxx_bridge_include_dir = cxx_bridge_include_dir(&config.project_root);

        let cxx_src_filter = |path: &PathBuf| {
            let ext = path.extension().unwrap_or_default();
            let is_target = CXX_SRC_EXTS.contains(&ext.to_str().unwrap_or_default());
            is_target
        };

        let cxx_header_filter = |path: &PathBuf| {
            let ext = path.extension().unwrap_or_default();
            let is_target = CXX_HEADER_EXTS.contains(&ext.to_str().unwrap_or_default());
            is_target
        };

        let cxx_srcs = collect_files(&cxx_bridge_dir, &cxx_src_filter)?;
        let cxx_headers = collect_files(&cxx_bridge_dir, &cxx_header_filter)?;
        let cxx_bridge_headers = collect_files(&cxx_bridge_include_dir, &cxx_header_filter)?;

        let lib_name = SanitizedString::from(&config.project.name);
        let lib = crate_target_dir(&config.project_root, target.to_str())
            .join(format!("lib{}.a", lib_base_name(&lib_name)));

        debug!("cxx_srcs: {:?}", cxx_srcs);
        debug!("cxx_headers: {:?}", cxx_headers);
        debug!("cxx_bridge_headers: {:?}", cxx_bridge_headers);
        debug!("lib: {:?}", lib);

        Ok(Artifacts {
            identifier: target.to_str().to_string(),
            srcs: cxx_srcs,
            headers: [cxx_headers, cxx_bridge_headers].concat(),
            libs: vec![lib],
        })
    }

    pub fn copy_to(
        &self,
        artifact_type: ArtifactType,
        dest: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let target_artifacts = match artifact_type {
            ArtifactType::Src => &self.srcs,
            ArtifactType::Header => &self.headers,
            ArtifactType::Lib => &self.libs,
        };

        if !dest.try_exists()? {
            debug!("Creating destination directory: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for src in target_artifacts {
            let file_name = src.file_name().unwrap();
            let ext = src.extension().unwrap().to_string_lossy().to_string();

            let dest = if artifact_type == ArtifactType::Lib {
                // Add `-prebuilt` suffix to the library name
                let lib_name = file_name.to_string_lossy().to_string().replace(
                    format!(".{}", ext).as_str(),
                    format!("-prebuilt.{}", ext).as_str(),
                );
                dest.join(lib_name)
            } else {
                dest.join(file_name)
            };

            debug!("Copying artifact: {:?} to {:?}", src, dest);
            fs::copy(src, dest)?;
        }

        Ok(())
    }
}
