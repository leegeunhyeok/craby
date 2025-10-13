use std::{fs, path::PathBuf};

use log::debug;

pub fn replace_cxx_header(signal_path: &PathBuf) -> Result<(), anyhow::Error> {
    debug!("Replacing cxx header in: {:?}", signal_path);
    let signals_h = fs::read_to_string(&signal_path)?;
    let signals_h = signals_h.replace("\"rust/cxx.h\"", "\"cxx.h\"");
    fs::write(&signal_path, signals_h)?;
    Ok(())
}

/// Workaround for the issue: https://github.com/dtolnay/cxx/issues/1574
pub fn replace_cxx_iter_template(cxx_path: &PathBuf) -> Result<(), anyhow::Error> {
    debug!("Replacing cxx iter template in: {:?}", cxx_path);
    let cxx_h = fs::read_to_string(&cxx_path)?;
    let cxx_h = cxx_h.replace(
        "using reference = typename std::add_lvalue_reference<T>::type;",
        "using reference = typename std::add_lvalue_reference<T>::type;\n  using element_type = T;",
    );
    fs::write(&cxx_path, cxx_h)?;
    Ok(())
}
