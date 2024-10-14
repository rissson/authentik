use axum::http::header::{HeaderMap, HeaderValue};
use http_auth_basic::Credentials;
use sea_orm::{DbErr, entity::prelude::*};

use crate::providers::oauth2::models::{OAuth2Provider, oauth2_provider};

pub(crate) struct ClientCredentials {
    client_id: String,
    client_secret: String,
}

pub(crate) fn extra_client_auth(
    headers: &HeaderMap,
    client_id: Option<&str>,
    client_secret: Option<&str>,
) -> Option<ClientCredentials> {
    let empty = HeaderValue::from_static("");
    let auth_header = match headers.get("HTTP_AUTHORIZATION").unwrap_or(&empty).to_str() {
        Ok(auth_header) => auth_header,
        Err(_) => return None,
    };

    if let Ok(creds) = Credentials::from_header(auth_header.to_owned()) {
        if creds.user_id.is_empty() || creds.password.is_empty() {
            return None;
        }
        return Some(ClientCredentials {
            client_id: creds.user_id,
            client_secret: creds.password,
        });
    }
    match (&client_id, &client_secret) {
        (Some(client_id), Some(client_secret)) if !client_id.is_empty() && !client_secret.is_empty() => {
            Some(ClientCredentials {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
            })
        }
        _ => None,
    }
}

pub(crate) async fn authenticate_provider(
    db: &DatabaseConnection,
    headers: &HeaderMap,
    client_id: Option<&str>,
    client_secret: Option<&str>,
) -> Result<Option<oauth2_provider::Model>, DbErr> {
    if let Some(creds) = extra_client_auth(headers, client_id, client_secret) {
        match OAuth2Provider::find()
            .filter(oauth2_provider::Column::ClientId.eq(&creds.client_id))
            .one(db)
            .await?
        {
            Some(provider) => {
                if provider.client_id != creds.client_id || provider.client_secret != creds.client_secret {
                    Ok(None)
                } else {
                    Ok(Some(provider))
                }
            }
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}
