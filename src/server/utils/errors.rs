use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug)]
pub struct ModelValidation {
    pub code: String,
    pub message: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("Entity already exists")]
    EntityAlreadyExists,

    #[error("Entity not found")]
    EntityNotFound,

    #[error("errors:?")]
    ModelValidation { errors: ModelValidation },

    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type ModelResult<T, E = ModelError> = std::result::Result<T, E>;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub error: Option<String>,
    pub description: Option<String>,
}

impl ErrorDetail {
    pub fn new<T: Into<String>>(error: T, description: T) -> Self {
        Self {
            error: Some(error.into()),
            description: Some(description.into()),
        }
    }

    pub fn with_reason<T: Into<String>>(error: T) -> Self {
        Self {
            error: Some(error.into()),
            description: None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{inner}\n{backtrace}")]
    WithBacktrace {
        inner: Box<Self>,
        backtrace: Box<std::backtrace::Backtrace>,
    },

    #[error("{0}")]
    Message(String),

    #[error(transparent)]
    Axum(#[from] http::Error),

    #[error(transparent)]
    JSON(serde_json::Error),

    #[error(transparent)]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    DB(#[from] sea_orm::DbErr),

    #[error("{0}")]
    Unauthorized(String),

    #[error("not found")]
    NotFound,

    #[error("{0}")]
    BadRequest(String),

    #[error("")]
    CustomError(StatusCode, ErrorDetail),

    #[error("Internal server error")]
    InternalServerError,

    #[error(transparent)]
    InvalidMethod(#[from] http::method::InvalidMethod),

    #[error(transparent)]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    #[error(transparent)]
    TaskJoinError(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Model(#[from] ModelError),

    #[error(transparent)]
    Any(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Self::JSON(val).bt()
    }
}

impl Error {
    pub fn wrap(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Any(Box::new(err))
    }

    pub fn msg(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Message(err.to_string())
    }

    pub fn string(s: &str) -> Self {
        Self::Message(s.to_string())
    }

    pub fn bt(self) -> Self {
        let backtrace = std::backtrace::Backtrace::capture();
        match backtrace.status() {
            std::backtrace::BacktraceStatus::Disabled | std::backtrace::BacktraceStatus::Unsupported => self,
            _ => Self::WithBacktrace {
                inner: Box::new(self),
                backtrace: Box::new(backtrace),
            },
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match &self {
            Self::WithBacktrace { inner, backtrace: _ } => {
                tracing::error!(error.msg = %inner, error.details = ?inner, "controller_error");
            }
            err => {
                tracing::error!(error.msg = %err, error.details = ?err, "controller_error");
            }
        }

        let public_facing_error = match self {
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorDetail::new("not_found", "Resource was not found"),
            ),
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetail::new("internal_server_error", "Internal Server Error"),
            ),
            Self::Unauthorized(err) => {
                tracing::warn!(err);
                (
                    StatusCode::UNAUTHORIZED,
                    ErrorDetail::new("unauthorized", "You do not have permission to access this resource"),
                )
            }
            Self::CustomError(status_code, data) => (status_code, data),
            Self::WithBacktrace { inner, backtrace: _ } => {
                // TODO:
                // println!("\n{}", inner.to_string().red().underline());
                return inner.into_response();
            }
            _ => (StatusCode::BAD_REQUEST, ErrorDetail::with_reason("Bad Request")),
        };

        (public_facing_error.0, axum::Json(public_facing_error.1)).into_response()
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
