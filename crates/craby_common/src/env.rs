pub fn is_rustup_installed() -> bool {
    std::process::Command::new("rustup")
        .arg("--version")
        .output()
        .is_ok()
}
