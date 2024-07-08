use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
pub(crate) enum RateLimiterError {
    Message(String)
}

impl fmt::Display for RateLimiterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimiterError::Message(e) => write!(f, "{}", e),
        }
    }
}

impl Error for RateLimiterError {}