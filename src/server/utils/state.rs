use axum::extract::FromRef;
use http::Uri;
use sea_orm::DatabaseConnection;

use crate::server::utils::backend::BackendClient;

#[derive(Clone)]
pub struct AppState {
    pub backend_uri: Uri,
    pub backend_client: BackendClient,
    pub db: DatabaseConnection,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> DatabaseConnection {
        state.db.clone()
    }
}
