use chrono::OutOfRangeError;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    TooManyRedirect(u64),
    Http(http::Error),
    Custom(Box<dyn std::error::Error + Send + Sync + 'static>),
    InvalidRedirectUrl(String),
    Internal(Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type Result<T> = core::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(inner) => write!(f, "Reqwest error: {:?}", inner),
            Error::Custom(inner) => write!(f, "Custom error: {:?}", inner),
            Error::Http(inner) => write!(f, "Build http error: {:?}", inner),
            Error::TooManyRedirect(redirect_count) => write!(
                f,
                "Too many redirect for this request: {redirect_count} time(s)."
            ),
            Error::InvalidRedirectUrl(url) => write!(f, "The redirect url is invalid: {url}"),
            Error::Internal(e) => write!(f, "Ergo internal error: {:?}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Self::Custom(value.into())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Self::Http(value)
    }
}

impl From<chrono::OutOfRangeError> for Error {
    fn from(value: OutOfRangeError) -> Self {
        Self::Internal(Box::new(value))
    }
}
