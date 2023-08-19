use inquire::{
    validator::{StringValidator, Validation},
    CustomUserError,
};
use regex::{Error, Regex};
use std::str::FromStr;

#[derive(Clone)]
pub struct PatternValidator {
    pattern: Regex,
    error_message: String,
}

impl PatternValidator {
    pub fn new(pattern: &str, error_message: &str) -> Result<Self, Error> {
        Ok(PatternValidator {
            pattern: Regex::from_str(pattern)?,
            error_message: error_message.into(),
        })
    }
}

impl StringValidator for PatternValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        if self.pattern.is_match(input) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid(self.error_message.clone().into()))
        }
    }
}