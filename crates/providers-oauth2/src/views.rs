use axum::Form;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Params {
    token: String,
}

#[derive(Serialize)]
struct Response {
    active: bool,
    scope: String,
    client_id: String,
}

async fn token_introspection(Form(params): Form<Params>) {

}
