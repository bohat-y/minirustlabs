use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
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
}

pub type Result<T> = std::result::Result<T, SubqError>;
