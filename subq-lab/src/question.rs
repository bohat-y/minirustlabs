use std::collections::BTreeMap;

use serde::Serialize;

use crate::{Result, SubqError};

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Noul(Noul),
    Choice(Choice),
    Score(Score),
}

#[derive(Debug, Clone, Serialize)]
pub struct Noul {
    instructions: String,
    criteria: NoulCriteria,
}

#[derive(Debug, Clone, Default, Serialize)]
struct NoulCriteria {
    #[serde(rename = "true", skip_serializing_if = "Option::is_none")]
    when_true: Option<String>,
    #[serde(rename = "false", skip_serializing_if = "Option::is_none")]
    when_false: Option<String>,
}

impl Noul {
    pub fn new(instructions: impl Into<String>) -> Self {
        Self {
            instructions: instructions.into(),
            criteria: NoulCriteria::default(),
        }
    }

    pub fn when_true(mut self, description: impl Into<String>) -> Self {
        self.criteria.when_true = Some(description.into());
        self
    }

    pub fn when_false(mut self, description: impl Into<String>) -> Self {
        self.criteria.when_false = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Choice {
    instructions: String,
    criteria: BTreeMap<String, Option<String>>,
}

impl Choice {
    pub fn new(instructions: impl Into<String>) -> Self {
        Self {
            instructions: instructions.into(),
            criteria: BTreeMap::new(),
        }
    }

    pub fn option(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.criteria.insert(name.into(), Some(description.into()));
        self
    }

    pub fn bare_option(mut self, name: impl Into<String>) -> Self {
        self.criteria.insert(name.into(), None);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Score {
    instructions: String,
    criteria: Vec<String>,
}

impl Score {
    pub fn new<L>(instructions: impl Into<String>, levels: L) -> Self
    where
        L: IntoIterator,
        L::Item: Into<String>,
    {
        Self {
            instructions: instructions.into(),
            criteria: levels.into_iter().map(|l| l.into()).collect(),
        }
    }
}

impl From<Noul> for Question {
    fn from(q: Noul) -> Self {
        Question::Noul(q)
    }
}

impl From<Choice> for Question {
    fn from(q: Choice) -> Self {
        Question::Choice(q)
    }
}

impl From<Score> for Question {
    fn from(q: Score) -> Self {
        Question::Score(q)
    }
}

impl Question {
    /// Checks the limits from the docs before anything is sent.
    pub fn validate(&self, name: &str) -> Result<()> {
        if name.is_empty() || name.chars().count() > 100 {
            return Err(SubqError::Invalid(format!(
                "question name `{name}` must be 1-100 characters"
            )));
        }
        match self {
            Question::Noul(_) => Ok(()),
            Question::Choice(c) if !(2..=255).contains(&c.criteria.len()) => Err(
                SubqError::Invalid(format!("choice `{name}` needs 2-255 options")),
            ),
            Question::Score(s) if !(2..=10).contains(&s.criteria.len()) => Err(SubqError::Invalid(
                format!("score `{name}` needs 2-10 levels"),
            )),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_like_the_docs() {
        let noul: Question = Noul::new("Should a nurse see this?")
            .when_true("Breathing trouble")
            .when_false("Stable symptoms")
            .into();
        assert_eq!(
            serde_json::to_value(&noul).unwrap(),
            json!({
                "type": "noul",
                "instructions": "Should a nurse see this?",
                "criteria": { "true": "Breathing trouble", "false": "Stable symptoms" }
            })
        );

        let choice: Question = Choice::new("Which team?")
            .option("nurse_line", "Needs a nurse today")
            .bare_option("front_desk")
            .into();
        assert_eq!(
            serde_json::to_value(&choice).unwrap(),
            json!({
                "type": "choice",
                "instructions": "Which team?",
                "criteria": { "nurse_line": "Needs a nurse today", "front_desk": null }
            })
        );

        let score: Question = Score::new("How soon?", ["Week", "Day", "Hour"]).into();
        assert_eq!(
            serde_json::to_value(&score).unwrap(),
            json!({
                "type": "score",
                "instructions": "How soon?",
                "criteria": ["Week", "Day", "Hour"]
            })
        );
    }

    #[test]
    fn rejects_bad_counts() {
        let one_option: Question = Choice::new("?").option("only", "one").into();
        assert!(one_option.validate("q").is_err());

        let one_level: Question = Score::new("?", ["just one"]).into();
        assert!(one_level.validate("q").is_err());
    }
}
