use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Item {
    pub id: Uuid,
    pub list_id: Uuid,
    pub title: String,
    pub checked: bool,
    pub pinned: bool,
}
