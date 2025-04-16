use nanoid::nanoid;
use std::fmt::Display;

use crate::domain::auth::entity::user::UserId;

#[readonly::make]
#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkId {
    pub value: String,
}

impl Default for LinkId {
    fn default() -> Self {
        Self { value: "".into() }
    }
}

impl Display for LinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl LinkId {
    pub fn generate() -> Self {
        Self{ value: nanoid!(4)}
    }
}

#[readonly::make]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Link {
    pub id: LinkId,
    // TODO: import UserId
    pub user_id: UserId,
    pub value: String,

    pub views: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_view: Option<chrono::DateTime<chrono::Utc>>,
}

impl Link {
    pub fn new(id: LinkId, user_id: UserId, value: String) -> Self {
        Self {
            id,
            user_id,
            value,
            views: 0,
            created_at: chrono::Utc::now(),
            last_view: None,
        }
    }

    pub fn increment_views(&mut self) {
        self.views += 1;
        self.last_view = chrono::Utc::now().into();
    }
}
