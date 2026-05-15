use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    Discovery(String),
    Monitoring(String),
    Security(String),
    Network(String),
    Visualization(String),
    Metadata(String),
    Persistence(String),
    Io(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Discovery(msg) => write!(f, "Discovery error: {msg}"),
            EngineError::Monitoring(msg) => write!(f, "Monitoring error: {msg}"),
            EngineError::Security(msg) => write!(f, "Security error: {msg}"),
            EngineError::Network(msg) => write!(f, "Network error: {msg}"),
            EngineError::Visualization(msg) => write!(f, "Visualization error: {msg}"),
            EngineError::Metadata(msg) => write!(f, "Metadata error: {msg}"),
            EngineError::Persistence(msg) => write!(f, "Persistence error: {msg}"),
            EngineError::Io(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;
