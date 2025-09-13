use std::{fs, path::PathBuf};

use craby_common::{
    config::CompleteCrabyConfig,
    constants::{cargo_lib_name, crate_target_dir, cxx_dir},
    utils::string::SanitizedString,
};
use log::debug;

use crate::{constants::toolchain::Target, utils::collect_files};

pub struct Artifacts {
    pub srcs: Vec<PathBuf>,
    pub headers: Vec<PathBuf>,
    pub libs: Vec<PathBuf>,
}

pub enum ArtifactType {
    Src,
    Header,
    Lib,
}

impl Artifacts {
    pub fn get_artifacts(
        config: &CompleteCrabyConfig,
        target: &Target,
    ) -> Result<Artifacts, anyhow::Error> {
        let cxx_dir = cxx_dir(&config.project_root, target.to_str());
        let cxx_srcs = collect_files(&cxx_dir, &["c", "cc"])?;
        let cxx_headers = collect_files(&cxx_dir, &["h", "hh"])?;

        let lib_name = SanitizedString::from(&config.project.name);
        let lib =
            crate_target_dir(&config.project_root, target.to_str()).join(cargo_lib_name(&lib_name));

        debug!("cxx_srcs: {:?}", cxx_srcs);
        debug!("cxx_headers: {:?}", cxx_headers);
        debug!("lib: {:?}", lib);

        Ok(Artifacts {
            srcs: cxx_srcs,
            headers: cxx_headers,
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

        if !fs::exists(dest)? {
            debug!("Creating destination directory: {:?}", dest);
            fs::create_dir_all(dest)?;
        }

        target_artifacts.iter().try_for_each(|src| {
            let dest = dest.join(src.file_name().unwrap());

            match artifact_type {
                // Skip copying cxx bridge source and header files to the destination
                // because generated cxx bridge sources are independent of the target platform
                ArtifactType::Src | ArtifactType::Header if fs::exists(&dest)? => {
                    return Ok(());
                }
                _ => (),
            }

            fs::copy(src, dest)?;
            Ok(())
        })
    }
}
