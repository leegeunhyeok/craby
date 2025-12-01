pub type Boolean = bool;
pub type Number = f64;
pub type String = std::string::String;
pub type ArrayBuffer = std::vec::Vec<u8>;
pub type Array<T> = std::vec::Vec<T>;
pub type Promise<T> = std::result::Result<T, anyhow::Error>;
pub type Void = ();

/// JavaScript-like Promise utilities.
pub mod promise {
    use super::Promise;

    /// Resolves a Promise with a value.
    /// Same as `Ok(v)`.
    pub fn resolve<T>(val: T) -> Promise<T> {
        Ok(val)
    }

    /// Rejects a Promise with an error message.
    /// Same as `Err(e)`.
    pub fn reject<T>(err: impl AsRef<str>) -> Promise<T> {
        Err(anyhow::anyhow!(err.as_ref().to_string()))
    }
}

/// JavaScript-like Nullable utilities.
///
/// Used to represent optional values.
///
/// ```typescript
/// let value: number | null = null;
/// let value: number | null = 123;
/// ```
///
pub struct Nullable<T> {
    val: Option<T>,
}

impl<T> Nullable<T> {
    /// Creates a new `Nullable` with an optional value.
    pub fn new(val: Option<T>) -> Self {
        Nullable { val }
    }

    /// Creates a new `Nullable` with a some value.
    pub fn some(val: T) -> Self {
        Nullable { val: Some(val) }
    }

    /// Creates a new `Nullable` with a none value.
    pub fn none() -> Self {
        Nullable { val: None }
    }

    /// Sets the value of the `Nullable`.
    pub fn value(mut self, val: T) -> Self {
        self.val = Some(val);
        self
    }

    /// Borrow the value reference of the `Nullable`.
    pub fn value_of(&self) -> Option<&T> {
        self.val.as_ref()
    }

    /// Takes the value out of the `Nullable`.
    pub fn into_value(self) -> Option<T> {
        self.val
    }
}
