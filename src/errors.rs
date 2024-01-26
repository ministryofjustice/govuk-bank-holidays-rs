use thiserror::Error;

/// Possible errors when handling dates and bank holidays
#[derive(Error, Debug)]
pub enum Error {
    // #[error("IO error")]
    // IO(#[from] std::io::Error),

    /// Parsing error – bank holiday data could not be parsed
    #[error("Parsing error")]
    Parsing(#[from] serde_json::Error),

    /// Request error – bank holiday data could not be loaded
    #[error("Request error")]
    Request(#[from] reqwest::Error),

    /// Date is invalid
    #[error("Invalid date")]
    InvalidDate,

    // #[error("{0}")]
    // Generic(&'static str),
}
