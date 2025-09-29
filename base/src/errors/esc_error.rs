use super::api_response_error::ApiResponseError;
use super::communication_error::CommunicationError;

/// Represents any error that can come back from an API operation
pub enum EscError {
    ApiResponse(Box<ApiResponseError>),
    Other(CommunicationError),
}

pub type Result<T> = std::result::Result<T, EscError>;

impl std::fmt::Display for EscError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> core::fmt::Result {
        match self {
            EscError::ApiResponse(err) => {
                write!(f, "Bad status code: {}", err)
            }
            EscError::Other(err) => {
                write!(f, "Unexpected failure: {}", err)
            }
        }
    }
}

impl std::fmt::Debug for EscError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> core::fmt::Result {
        match self {
            EscError::ApiResponse(err) => {
                write!(f, "Bad status code: {:?}", err)
            }
            EscError::Other(err) => {
                write!(f, "Unexpected failure: {:?}", err)
            }
        }
    }
}

impl std::error::Error for EscError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EscError::ApiResponse(_) => None,
            EscError::Other(err) => Some(err),
        }
    }
}

impl EscError {
    /// Returns the operation error as a Result. The purpose of this is to
    /// make it possible to use the question mark to fetch the actual API
    /// error, and pass back the other error if it was something else.
    #[allow(clippy::result_large_err)]
    pub fn api_response(self) -> std::result::Result<Box<ApiResponseError>, EscError> {
        match self {
            EscError::ApiResponse(err) => Ok(err),
            EscError::Other(_) => Err(self),
        }
    }
}

impl From<CommunicationError> for EscError {
    fn from(err: CommunicationError) -> Self {
        Self::Other(err)
    }
}

impl From<ApiResponseError> for EscError {
    fn from(err: ApiResponseError) -> Self {
        Self::ApiResponse(Box::new(err))
    }
}

impl PartialEq for EscError {
    fn eq(&self, other: &EscError) -> bool {
        match self {
            EscError::ApiResponse(err) => match other {
                EscError::ApiResponse(other_err) => err == other_err,
                _ => false,
            },
            EscError::Other(_) => false,
        }
    }
}
