use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("entry not found: {0}")]
    EntryNotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type AppResult<T> = Result<T, AppError>;
