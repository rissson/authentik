pub mod models;

mod constants;
mod id_token;
mod utils;
use authentik_server_utils::state::AppState;
use axum::{Router, routing::post};
mod views;

pub async fn make_router() -> Router<AppState> {
    Router::new().route("/application/o/introspect/", post(views::token_introspection))
}
