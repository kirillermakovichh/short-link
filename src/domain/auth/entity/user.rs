use std::borrow::Cow;
use std::fmt::Display;

use uuid::Uuid;
use validator::{ValidationError, ValidationErrors};


#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error("invalid user id: `{0}`")]
    InvalidUserId(String, #[source] ValidationErrors),
}

#[readonly::make]
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
pub struct UserId {
    pub value: Uuid,
}

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl UserId {
    pub fn generate() -> Self {
        Uuid::new_v4().into()
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self { value: uuid }
    }
}

impl TryFrom<String> for UserId {
    type Error = UserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Uuid::parse_str(&value)
            .map_err(|err| {
                let mut errors = ValidationErrors::new();
                errors.add(
                    "value",
                    ValidationError::new("invalid_id").with_message(Cow::from(err.to_string())),
                );
                UserError::InvalidUserId(value, errors)
            })?
            .into())
    }
}

impl TryFrom<&str> for UserId {
    type Error = UserError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}