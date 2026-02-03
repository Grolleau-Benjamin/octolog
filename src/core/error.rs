use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("config error: {0}")]
    Config(String),

    #[error("no ports specified")]
    NoPortsFound,

    #[error("invalid port format: {0}")]
    PortInvalidFormat(String),

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("scan error: {0}")]
    Scan(String),
}
