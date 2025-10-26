/// The context of the Craby Module.
pub struct Context {
    /// This is a unique identifier(pointer address) for the current TurboModule instance.
    ///
    /// Used by
    /// - Emitting signals to specific TurboModule instances.
    pub id: usize,
    /// This is the path to the application's data directory.
    ///
    /// **WARNING**: Only access files within this directory, do not write to other directories.
    pub data_path: String,
}

impl Context {
    pub fn new(id: usize, data_path: &str) -> Self {
        Context {
            id,
            data_path: data_path.to_string(),
        }
    }
}
