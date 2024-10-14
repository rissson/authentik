use std::collections::HashMap;

use derivative::Derivative;
use either::Either;
use serde::{Deserialize, Serialize};

use crate::constants;

/// Mode after which `sub` attribute is generated, for compatibility reasons
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum SubModes {
    /// Based on the hashed user ID
    HashedUserID,
    /// Based on the user ID
    UserID,
    /// Based on the user UUID
    UserUUID,
    /// Based on the username
    UserUsername,
    /// Based on the user email. This is recommended over the UPN method
    UserEmail,
    /// Based on the user's UPN, only works if user has a `upn` attribute set.
    /// Use this method only if you have different UPN and mail domains.
    UserUPN,
}

/// The primary extension that OpenID Connect makes to OAuth 2.0 to enable End-Users to be
/// Authenticated is the ID Token data structure. The ID Token is a security token that contains
/// Claims about the Authentication of an End-User by an Authorization Server when using a Client,
/// and potentially other requested Claims. The ID Token is represented as a JSON Web Token (JWT).
/// See https://openid.net/specs/openid-connect-core-1_0.html#IDToken
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Derivative, Serialize, Deserialize)]
#[derivative(Default)]
pub(crate) struct IDToken {
    /// Issuer, https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.1
    iss: Option<String>,
    /// Subject, https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.2
    sub: Option<String>,
    /// Audience, https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.3
    aud: Option<Either<String, Vec<String>>>,
    /// Expiration time, https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.4
    exp: Option<u64>,
    /// Issued at, https://www.rfc-editor.org/rfc/rfc7519.html#section-4.1.6
    iat: Option<u64>,
    /// Time when the authentication occurred,
    /// //openid.net/specs/openid-connect-core-1_0.html#IDToken
    auth_time: Option<u64>,
    /// Authentication Context Class Reference,
    /// https://openid.net/specs/openid-connect-core-1_0.html#IDToken
    #[derivative(Default(value = "Some(constants::ACR_AUTHENTIK_DEFAULT.to_owned())"))]
    acr: Option<String>,
    /// Authentication Methods References,
    /// https://openid.net/specs/openid-connect-core-1_0.html#IDToken
    amr: Option<Vec<String>>,
    /// Code hash value, http://openid.net/specs/openid-connect-core-1_0.html
    c_hash: Option<String>,
    /// Value used to associate a Client session with an ID Token,
    /// http://openid.net/specs/openid-connect-core-1_0.html
    nonce: Option<String>,
    /// Access Token hash value, http://openid.net/specs/openid-connect-core-1_0.html
    at_hash: Option<String>,

    claims: HashMap<String, String>,
}
