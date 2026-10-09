use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct List {
    pub(crate) id: Uuid,
    pub(crate) title: String,
    pub(crate) pinned: bool,
}
