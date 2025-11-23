use craby_build::constants::toolchain::{Target, DEFAULT_ANDROID_TARGETS, DEFAULT_IOS_TARGETS};
use craby_common::config::CompleteConfig;
use owo_colors::OwoColorize;

pub fn get_build_targets(config: &CompleteConfig) -> Result<Vec<Target>, anyhow::Error> {
    let android =
        get_targets_with_defaults(config.android.targets.as_ref(), &DEFAULT_ANDROID_TARGETS)?;
    let ios = get_targets_with_defaults(config.ios.targets.as_ref(), &DEFAULT_IOS_TARGETS)?;

    Ok([android, ios].concat())
}

pub fn print_build_targets(targets: &[Target]) {
    for (idx, target) in targets.iter().enumerate() {
        let is_last = idx == targets.len() - 1;
        let branch = if is_last { "└─" } else { "├─" };
        let platform = match target {
            Target::Android(_) => format!("{}", "(Android)".green()),
            Target::Ios(_) => format!("{}", "(iOS)".blue()),
        };
        println!("{} {} {}", branch, platform, target.to_str().dimmed());
    }
}

fn get_targets_with_defaults(
    config_targets: Option<&Vec<String>>,
    defaults: &[Target],
) -> Result<Vec<Target>, anyhow::Error> {
    match config_targets {
        Some(targets) => targets
            .iter()
            .map(|s| Target::try_from(s.as_str()))
            .collect(),
        None => Ok(defaults.to_vec()),
    }
}
