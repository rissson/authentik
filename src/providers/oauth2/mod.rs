pub mod models;

mod constants;
mod id_token;
mod utils;
use axum::{Router, routing::post};

use crate::server::utils::state::AppState;
mod views;

pub async fn make_router() -> Router<AppState> {
    Router::new().route("/application/o/introspect/", post(views::token_introspection))
}
