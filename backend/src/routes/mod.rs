// API route handlers

pub mod audit;
pub mod devices;
pub mod policies;
pub mod sessions;

use axum::Router;
use crate::AppState;

/// Creates API router with all routes
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .merge(audit::routes())
        .nest("/devices", devices::routes())
        .nest("/devices", policies::routes())
        .nest("/sessions", sessions::routes())
}
