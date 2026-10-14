use crate::{Result, SubqError};

const DEFAULT_URL: &str = "https://api.subq.ai/v1";

#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl Client {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: DEFAULT_URL.to_string(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("SUBQ_API_KEY").map_err(|_| SubqError::MissingApiKey)?;
        Ok(Self::new(api_key))
    }

    #[cfg(feature = "openai")]
    pub fn openai(&self) -> async_openai::Client<async_openai::config::OpenAIConfig> {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_base(&self.base_url)
            .with_api_key(&self.api_key);
        async_openai::Client::with_config(config).with_http_client(self.http.clone())
    }
}
