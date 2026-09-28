use utoipa::OpenApi;

use crate::model::{Item, List};
#[allow(clippy::wildcard_imports)]
use crate::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_lists,
        get_list,
        create_list,
        delete_list,
        patch_list,
        get_item,
        create_item,
        delete_item,
        patch_item,
    ),
    components(
        schemas(
            List,
            Item
        )
    ),
    tags(
        (name = "Lists", description = "List management endpoints"),
        (name = "Items", description = "Item management endpoints")
    ),
    info(
        title = "Lists and Items API",
        version = "1.0.0",
        description = "REST API for managing lists and their items"
    )
)]
pub struct ApiDoc;
