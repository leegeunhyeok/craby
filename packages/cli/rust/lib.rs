#![deny(clippy::all)]

use log::{debug, error, info, trace, warn, LevelFilter};

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn setup(level_filter: Option<String>) {
    let level_filter = level_filter.and_then(|l| match l.as_str() {
        "trace" => Some(LevelFilter::Trace),
        "debug" => Some(LevelFilter::Debug),
        "info" => Some(LevelFilter::Info),
        "warn" => Some(LevelFilter::Warn),
        "error" => Some(LevelFilter::Error),
        _ => None,
    });

    craby_cli::logger::init(level_filter);
    debug!("Setup with level filter: {:?}", level_filter);
}

#[napi(object)]
pub struct InitOptions {
    pub project_root: String,
    pub template_base_path: String,
    pub library_name: String,
}

#[napi]
pub fn init(opts: InitOptions) -> napi::Result<()> {
    let opts = craby_cli::commands::init::InitOptions {
        project_root: opts.project_root.into(),
        template_base_path: opts.template_base_path.into(),
        lib_name: opts.library_name,
    };

    match craby_cli::commands::init::r#impl(opts) {
        Err(e) => Err(napi::Error::new(
            napi::Status::GenericFailure,
            e.to_string(),
        )),
        _ => Ok(()),
    }
}

#[napi(object)]
pub struct CodegenOptions {
    pub project_root: String,
    pub library_name: String,
    pub java_package_name: String,
    pub schemas: Vec<String>,
}

#[napi]
pub fn codegen(opts: CodegenOptions) -> napi::Result<()> {
    let opts = craby_cli::commands::codegen::CodegenOptions {
        project_root: opts.project_root.into(),
        lib_name: opts.library_name,
        java_package_name: opts.java_package_name,
        schemas: opts.schemas,
    };

    match craby_cli::commands::codegen::r#impl(opts) {
        Err(e) => Err(napi::Error::new(
            napi::Status::GenericFailure,
            e.to_string(),
        )),
        _ => Ok(()),
    }
}

#[napi(object)]
pub struct BuildOptions {
    pub project_root: String,
    pub library_name: String,
}

#[napi]
pub fn build(opts: BuildOptions) -> napi::Result<()> {
    let opts = craby_cli::commands::build::BuildOptions {
        project_root: opts.project_root.into(),
        lib_name: opts.library_name,
    };

    match craby_cli::commands::build::r#impl(opts) {
        Err(e) => Err(napi::Error::new(
            napi::Status::GenericFailure,
            e.to_string(),
        )),
        _ => Ok(()),
    }
}

#[napi(object)]
pub struct ShowOptions {
    pub project_root: String,
    pub library_name: String,
    pub schemas: Vec<String>,
}

#[napi]
pub fn show(opts: ShowOptions) -> napi::Result<()> {
    let opts = craby_cli::commands::show::ShowOptions {
        project_root: opts.project_root.into(),
        lib_name: opts.library_name,
        schemas: opts.schemas,
    };

    match craby_cli::commands::show::r#impl(opts) {
        Err(e) => Err(napi::Error::new(
            napi::Status::GenericFailure,
            e.to_string(),
        )),
        _ => Ok(()),
    }
}

#[napi(object)]
pub struct CleanOptions {
    pub project_root: String,
}

#[napi]
pub fn clean(opts: CleanOptions) -> napi::Result<()> {
    let opts = craby_cli::commands::clean::CleanOptions {
        project_root: opts.project_root.into(),
    };

    match craby_cli::commands::clean::r#impl(opts) {
        Err(e) => Err(napi::Error::new(
            napi::Status::GenericFailure,
            e.to_string(),
        )),
        _ => Ok(()),
    }
}

#[napi]
pub fn trace(message: String) {
    trace!("{}", message);
}

#[napi]
pub fn debug(message: String) {
    debug!("{}", message);
}

#[napi]
pub fn info(message: String) {
    info!("{}", message);
}

#[napi]
pub fn warn(message: String) {
    warn!("{}", message);
}

#[napi]
pub fn error(message: String) {
    error!("{}", message);
}
