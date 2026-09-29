//! HTTP routing.

mod health;
mod orders;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .nest("/api/v1", v1_router())
}

fn v1_router() -> Router<AppState> {
    Router::new().route("/orders", post(orders::create_order)).route(
        "/orders/{id}",
        get(orders::get_order),
    )
}
