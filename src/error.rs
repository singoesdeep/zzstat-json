use thiserror::Error;

/// Errors that can occur when loading or processing JSON stat configurations.
#[derive(Debug, Error)]
pub enum YamlStatError {
    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// Stat resolution error
    #[error("Stat resolution error: {0}")]
    ResolutionError(#[from] zzstat::StatError),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Missing dependency
    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    /// Invalid transform type
    #[error("Invalid transform type: {0}")]
    InvalidTransformType(String),
}
