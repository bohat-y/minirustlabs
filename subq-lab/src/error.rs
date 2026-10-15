use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SubqError {
    #[error("SUBQ_API_KEY is not set")]
    MissingApiKey,

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("api error {status} ({error_type}): {message}")]
    Api {
        status: StatusCode,
        error_type: String,
        message: String,
    },

    #[error("invalid request: {0}")]
    Invalid(String),

    #[error("no {kind} answer named `{name}`")]
    MissingAnswer { name: String, kind: &'static str },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SubqError>;

/// Decisions errors look like `{ "detail": { "error_type": ..., "message": ... } }`.
#[derive(Deserialize)]
struct ErrorBody {
    detail: ErrorDetail,
}

#[derive(Deserialize)]
struct ErrorDetail {
    error_type: String,
    message: String,
}

impl SubqError {
    /// Builds an `Api` error from a failed response. Falls back to the raw
    /// body when it isn't the `detail` shape (e.g. an HTML 502 page).
    pub(crate) fn from_response(status: StatusCode, body: &str) -> Self {
        match serde_json::from_str::<ErrorBody>(body) {
            Ok(ErrorBody { detail }) => SubqError::Api {
                status,
                error_type: detail.error_type,
                message: detail.message,
            },
            Err(_) => SubqError::Api {
                status,
                error_type: "unknown".to_string(),
                message: body.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_detail_errors() {
        let body = r#"{ "detail": { "error_type": "validation_error", "message": "bad image" } }"#;
        match SubqError::from_response(StatusCode::UNPROCESSABLE_ENTITY, body) {
            SubqError::Api {
                status,
                error_type,
                message,
            } => {
                assert_eq!(status, 422);
                assert_eq!(error_type, "validation_error");
                assert_eq!(message, "bad image");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn keeps_non_json_bodies() {
        match SubqError::from_response(StatusCode::BAD_GATEWAY, "<html>502</html>") {
            SubqError::Api {
                error_type,
                message,
                ..
            } => {
                assert_eq!(error_type, "unknown");
                assert_eq!(message, "<html>502</html>");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }
}
