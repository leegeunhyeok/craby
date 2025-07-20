use inquire::{
    validator::{ErrorMessage, StringValidator, Validation},
    CustomUserError,
};

#[derive(Clone)]
pub struct CrateNameValidator;

impl StringValidator for CrateNameValidator {
    fn validate(&self, v: &str) -> Result<Validation, CustomUserError> {
        let mut chars = v.chars();

        if v.is_empty() {
            return Ok(Validation::Invalid(ErrorMessage::Custom(
                "Crate name cannot be empty".to_string(),
            )));
        }

        if !chars.all(|c| c.is_ascii_lowercase() || c == '_') {
            return Ok(Validation::Invalid(ErrorMessage::Custom(
                "Crate name must contain only lowercase letters and underscores".to_string(),
            )));
        }

        Ok(Validation::Valid)
    }
}
