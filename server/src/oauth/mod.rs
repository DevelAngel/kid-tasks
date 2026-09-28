mod config;
mod handlers;
mod store;

pub use config::McpClientsConfig;
pub use handlers::{
    approve, auth_server, authorize, gen_access_token, protected_resource, validate_access_token,
};
pub use store::{ClientName, McpOAuthStore, OnBehalfOf, Prefix};
