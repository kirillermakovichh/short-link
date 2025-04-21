use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserNoPassword {
    pub id: i32,
    pub name: String,
    pub email: String,
}

impl UserNoPassword {
    pub fn new(id: i32, name: String, email: String) -> Self {
        Self { id, name, email }
    }
}
