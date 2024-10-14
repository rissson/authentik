use authentik_orm_utils::expiring_model::ExpiringModel;
use authentik_server_utils::errors::Result;
use axum::{Form, Json, extract::State, http::header::HeaderMap, response::IntoResponse};
use sea_orm::{DatabaseConnection, entity::prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    id_token::IDToken,
    models::{AccessToken, access_token, RefreshToken, refresh_token},
    utils::authenticate_provider,
};

#[derive(Deserialize)]
pub(crate) struct Params {
    token: String,
    client_id: Option<String>,
    client_secret: Option<String>,
}

#[serde_with::skip_serializing_none]
#[derive(Default, Serialize)]
struct Response {
    active: bool,
    scope: Option<String>,
    client_id: Option<String>,
    #[serde(flatten)]
    id_token: Option<IDToken>,
}

impl Response {
    fn error() -> Self {
        Self {
            active: false,
            ..Default::default()
        }
    }
}

#[axum::debug_handler]
pub(crate) async fn token_introspection(
    State(db): State<DatabaseConnection>,
    headers: HeaderMap,
    Form(params): Form<Params>,
) -> Result<impl IntoResponse> {
    if let Some(provider) = authenticate_provider(
        &db,
        &headers,
        params.client_id.as_deref(),
        params.client_secret.as_deref(),
    )
    .await?
    {
        if let Some(access_token) = AccessToken::find()
            .filter(access_token::Column::Token.eq(&params.token))
            .filter(access_token::Column::ProviderId.eq(provider.provider_ptr_id))
            .one(&db)
            .await?
        {
            if let Ok(id_token) = access_token.get_id_token() {
                return Ok(Json(Response {
                    active: !access_token.is_expired() && !access_token.revoked,
                    scope: Some(access_token.get_scope().join(" ")),
                    client_id: Some(provider.client_id),
                    id_token: Some(id_token),
                }));
            }
            return Ok(Json(Response::error()))
        }
        if let Some(refresh_token) = RefreshToken::find()
            .filter(refresh_token::Column::Token.eq(&params.token))
            .filter(refresh_token::Column::ProviderId.eq(provider.provider_ptr_id))
            .one(&db)
            .await?
        {
            if let Ok(id_token) = refresh_token.get_id_token() {
                return Ok(Json(Response {
                    active: !refresh_token.is_expired() && !refresh_token.revoked,
                    scope: Some(refresh_token.get_scope().join(" ")),
                    client_id: Some(provider.client_id),
                    id_token: Some(id_token),
                }));
            }
            return Ok(Json(Response::error()))
        }
    }
    Ok(Json(Response::error()))
}

#[cfg(test)]
mod tests {}
