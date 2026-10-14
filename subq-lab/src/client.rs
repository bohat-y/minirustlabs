use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;
use serde_json::Value;

use crate::{Decision, Question, Result, SubqError};

const DEFAULT_URL: &str = "https://api.subq.ai/v1";
const DEFAULT_DECISION_MODEL: &str = "subq-snap-preview";

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

// Written by hand so `{:?}` never prints the API key.
impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .finish()
    }
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

    /// Starts a decision about `state`: a `&str`, `String`, or any `serde_json::Value`.
    pub fn decide(&self, state: impl Into<Value>) -> DecisionRequest<'_> {
        DecisionRequest {
            client: self,
            body: DecisionBody {
                model: DEFAULT_DECISION_MODEL.to_string(),
                state: state.into(),
                questions: BTreeMap::new(),
            },
        }
    }

    #[cfg(feature = "openai")]
    pub fn openai(&self) -> async_openai::Client<async_openai::config::OpenAIConfig> {
        let config = async_openai::config::OpenAIConfig::new()
            .with_api_base(&self.base_url)
            .with_api_key(&self.api_key);
        async_openai::Client::with_config(config).with_http_client(self.http.clone())
    }
}

/// The JSON body for `POST /v1/systemone`.
#[derive(Debug, Serialize)]
struct DecisionBody {
    model: String,
    state: Value,
    questions: BTreeMap<String, Question>,
}

/// Builder returned by [`Client::decide`]. Add questions, then `.send().await`.
#[derive(Debug)]
pub struct DecisionRequest<'a> {
    client: &'a Client,
    body: DecisionBody,
}

impl DecisionRequest<'_> {
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.body.model = model.into();
        self
    }

    pub fn question(mut self, name: impl Into<String>, question: impl Into<Question>) -> Self {
        self.body.questions.insert(name.into(), question.into());
        self
    }

    pub async fn send(self) -> Result<Decision> {
        self.validate()?;

        let res = self
            .client
            .http
            .post(format!("{}/systemone", self.client.base_url))
            .bearer_auth(&self.client.api_key)
            .json(&self.body)
            .send()
            .await?;

        let status = res.status();
        if status.is_success() {
            return Ok(res.json().await?);
        }
        let body = res.text().await.unwrap_or_default();
        Err(SubqError::from_response(status, &body))
    }

    fn validate(&self) -> Result<()> {
        let count = self.body.questions.len();
        if !(1..=16).contains(&count) {
            return Err(SubqError::Invalid(format!(
                "a decision needs 1-16 questions, got {count}"
            )));
        }
        for (name, question) in &self.body.questions {
            question.validate(name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Choice, Noul, Score};
    use serde_json::json;

    #[test]
    fn builds_the_docs_request() {
        let client = Client::new("sk-test".to_string());
        let req = client
            .decide("Patient with asthma, short of breath.")
            .question(
                "escalate",
                Noul::new("Should a nurse see this?")
                    .when_true("Breathing trouble")
                    .when_false("Stable symptoms"),
            )
            .question(
                "queue",
                Choice::new("Which team?")
                    .option("nurse_line", "Needs a nurse today")
                    .bare_option("front_desk"),
            )
            .question("acuity", Score::new("How soon?", ["Week", "Day", "Hour"]));

        assert!(req.validate().is_ok());
        assert_eq!(
            serde_json::to_value(&req.body).unwrap(),
            json!({
                "model": "subq-snap-preview",
                "state": "Patient with asthma, short of breath.",
                "questions": {
                    "escalate": {
                        "type": "noul",
                        "instructions": "Should a nurse see this?",
                        "criteria": { "true": "Breathing trouble", "false": "Stable symptoms" }
                    },
                    "queue": {
                        "type": "choice",
                        "instructions": "Which team?",
                        "criteria": { "nurse_line": "Needs a nurse today", "front_desk": null }
                    },
                    "acuity": {
                        "type": "score",
                        "instructions": "How soon?",
                        "criteria": ["Week", "Day", "Hour"]
                    }
                }
            })
        );
    }

    #[test]
    fn rejects_zero_questions() {
        let client = Client::new("sk-test".to_string());
        assert!(client.decide("anything").validate().is_err());
    }

    #[test]
    fn debug_hides_the_key() {
        let client = Client::new("sk-secret".to_string());
        assert!(!format!("{client:?}").contains("sk-secret"));
    }
}
