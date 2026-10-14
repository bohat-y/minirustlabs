mod client;
mod error;
mod question;

pub use client::Client;
pub use error::{Result, SubqError};
pub use question::{Choice, Noul, Question, Score};

#[cfg(feature = "openai")]
pub use async_openai;
