use std::collections::HashMap;

use serde::Deserialize;

use crate::{Result, SubqError};

/// The full response from `POST /v1/systemone`.
#[derive(Debug, Clone, Deserialize)]
pub struct Decision {
    pub model: String,
    pub answers: HashMap<String, Answer>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// One answer. Picks the variant from the `"type"` field.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
    Noul(NoulAnswer),
    Choice(ChoiceAnswer),
    Score(ScoreAnswer),
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct NoulAnswer {
    /// Probability of yes, 0 to 1.
    pub noul: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceAnswer {
    pub choice: String,
    pub probabilities: HashMap<String, f64>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoreAnswer {
    /// Probability-weighted level, e.g. 1.86 between levels 1 and 2.
    pub score: f64,
    /// Level index ("0", "1", ...) to the label you sent.
    pub legend: HashMap<String, String>,
    pub probabilities: HashMap<String, f64>,
    pub confidence: f64,
}

impl ScoreAnswer {
    /// Label of the level nearest to `score`.
    pub fn nearest_label(&self) -> Option<&str> {
        let level = self.score.round() as u32;
        self.legend.get(&level.to_string()).map(String::as_str)
    }
}

impl Decision {
    pub fn noul(&self, name: &str) -> Result<f64> {
        match self.answers.get(name) {
            Some(Answer::Noul(a)) => Ok(a.noul),
            _ => Err(missing(name, "noul")),
        }
    }

    pub fn choice(&self, name: &str) -> Result<&ChoiceAnswer> {
        match self.answers.get(name) {
            Some(Answer::Choice(a)) => Ok(a),
            _ => Err(missing(name, "choice")),
        }
    }

    pub fn score(&self, name: &str) -> Result<&ScoreAnswer> {
        match self.answers.get(name) {
            Some(Answer::Score(a)) => Ok(a),
            _ => Err(missing(name, "score")),
        }
    }
}

fn missing(name: &str, kind: &'static str) -> SubqError {
    SubqError::MissingAnswer {
        name: name.to_string(),
        kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The response example from the Decisions API docs.
    const DOCS_RESPONSE: &str = r#"{
        "model": "subq-snap-preview",
        "answers": {
            "escalate": { "type": "noul", "noul": 0.92 },
            "queue": {
                "type": "choice",
                "choice": "nurse_line",
                "probabilities": { "nurse_line": 1, "primary_care": 0, "pharmacy": 0, "front_desk": 0 },
                "confidence": 1
            },
            "acuity": {
                "type": "score",
                "score": 1.86,
                "legend": { "0": "Within a week", "1": "Within a day", "2": "Within an hour" },
                "probabilities": { "0": 0, "1": 0.13, "2": 0.87 },
                "confidence": 0.79
            }
        },
        "usage": { "input_tokens": 477, "output_tokens": 0 }
    }"#;

    #[test]
    fn parses_the_docs_example() {
        let d: Decision = serde_json::from_str(DOCS_RESPONSE).unwrap();

        assert_eq!(d.noul("escalate").unwrap(), 0.92);
        assert_eq!(d.choice("queue").unwrap().choice, "nurse_line");

        let acuity = d.score("acuity").unwrap();
        assert_eq!(acuity.score, 1.86);
        assert_eq!(acuity.nearest_label(), Some("Within an hour"));

        assert_eq!(d.usage.input_tokens, 477);
    }

    #[test]
    fn wrong_name_or_type_is_an_error() {
        let d: Decision = serde_json::from_str(DOCS_RESPONSE).unwrap();

        assert!(d.noul("nope").is_err()); // no such answer
        assert!(d.noul("queue").is_err()); // exists, but it's a choice
    }
}
