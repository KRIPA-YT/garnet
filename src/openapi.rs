use utoipa::OpenApi;

use crate::items::model::Item;
use crate::lists::model::List;
#[allow(clippy::wildcard_imports)]
use crate::routes::auth::*;
#[allow(clippy::wildcard_imports)]
use crate::routes::item::*;
#[allow(clippy::wildcard_imports)]
use crate::routes::list::*;

use utoipa::{
    Modify,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            // Authorization: Bearer <token>
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            );

            // Authorization: Basic <base64(email:password)>
            components.add_security_scheme(
                "basic_auth",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Basic).build()),
            );
        }
    }
}

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
        register,
        login,
        refresh,
    ),
    components(
        schemas(
            List,
            Item
        )
    ),
    tags(
        (name = "Lists", description = "List management endpoints"),
        (name = "Items", description = "Item management endpoints"),
        (name = "Auth", description = "Authentication endpoints")
    ),
    info(
        title = "Lists and Items API",
        version = "1.0.0",
        description = "REST API for managing lists and their items"
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;
