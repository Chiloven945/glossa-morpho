use thiserror::Error;

#[expect(
    dead_code,
    reason = "Reserved for future shared command error handling."
)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("entry not found: {0}")]
    EntryNotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

#[expect(
    dead_code,
    reason = "Reserved for future shared command result handling."
)]
pub type AppResult<T> = Result<T, AppError>;
