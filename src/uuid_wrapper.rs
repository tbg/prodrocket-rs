use std;
use std::str::FromStr;

use serde::{Serialize, Serializer};

use rocket::request::FromFormValue;
use rocket::http::RawStr;

use uuid;

/// A wrapper around uuid::Uuid.
pub struct Uuid(uuid::Uuid);

#[derive(Debug)]
/// Errors occurring when parsing a Uuid from a String.
pub enum UuidParseError {
    StrError(super::std::string::ParseError),
    UuidError(uuid::ParseError),
}

impl<'v> FromFormValue<'v> for Uuid {
    type Error = UuidParseError;
    fn from_form_value(form_value: &'v RawStr) -> std::result::Result<Self, Self::Error> {
        let s = form_value
            .parse::<String>()
            .map_err(UuidParseError::StrError)?;
        uuid::Uuid::from_str(&s)
            .map_err(UuidParseError::UuidError)
            .map(Uuid)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.hyphenated().to_string())
    }
}
