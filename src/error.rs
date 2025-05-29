use async_openai::error::OpenAIError;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, AgentError>;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Generic error: {0}")]
    Generic(String),

    #[error("OpenAI error: {0}")]
    OpenAIError(#[from] OpenAIError),

    #[error("HTTP error: {0}")]
    HTTPError(#[from] reqwest::Error),
}
