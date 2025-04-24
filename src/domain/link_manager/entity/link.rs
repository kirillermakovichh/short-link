use nanoid::nanoid;
use std::fmt::Display;
use utoipa::ToSchema;

#[readonly::make]
#[derive(Debug, Eq, PartialEq, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct LinkId {
    pub value: String,
}

impl Display for LinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl LinkId {
    pub fn generate() -> Self {
        Self { value: nanoid!(4) }
    }

    pub fn from_string(id: String) -> Self {
        Self { value: id }
    }
}

#[readonly::make]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Link {
    pub id: LinkId,
    pub user_id: i32,
    pub redirect_url: String,
    pub label: String,

    pub views: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_view: Option<chrono::DateTime<chrono::Utc>>,
}

impl Link {
    pub fn new(id: LinkId, user_id: i32, redirect_url: String, label: String) -> Self {
        Self {
            id,
            user_id,
            redirect_url,
            label,
            views: 0,
            created_at: chrono::Utc::now(),
            last_view: None,
        }
    }

    pub fn from_parts(
        id: LinkId,
        user_id: i32,
        redirect_url: String,
        label: String,
        views: i64,
        created_at: chrono::DateTime<chrono::Utc>,
        last_view: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            id,
            user_id,
            redirect_url,
            label,
            views,
            created_at,
            last_view,
        }
    }
}
