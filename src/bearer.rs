use actix_web::{http, FromRequest, HttpRequest, ResponseError};
use reqwest::StatusCode;
use std::{
    fmt::{self},
    fs,
    future::{ready, Ready},
    io,
    path::PathBuf,
};

use crate::utils::read_token;

#[derive(Debug, Clone, PartialEq)]
pub struct BearerConfig {
    path: Option<PathBuf>,
}
impl BearerConfig {
    pub fn new(path: Option<PathBuf>) -> Result<Self, io::Error> {
        let path = path.into();
        // just to fail early in case the token file is unreadable
        let path = match path {
            Some(path) => {
                read_token(&mut fs::File::open(&path)?)?;
                Some(path)
            }
            None => None,
        };
        Ok(Self { path })
    }
}
impl Default for BearerConfig {
    fn default() -> Self {
        Self { path: None }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BearerResponseError {
    ConfigRead,
    ConfigMissing,
    MissingAuthHeader,
    BearerMissmatch,
}
impl fmt::Display for BearerResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = self.status_code();
        match status.canonical_reason() {
            Some(reason) => write!(f, "{} {}", status.as_u16(), reason),
            None => write!(f, "{}", status),
        }
    }
}
impl ResponseError for BearerResponseError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConfigMissing => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ConfigRead => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingAuthHeader => StatusCode::UNAUTHORIZED,
            Self::BearerMissmatch => StatusCode::UNAUTHORIZED,
        }
    }
}

pub struct Bearer;
impl FromRequest for Bearer {
    type Error = BearerResponseError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = BearerConfig;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        fn from_request_inner(req: &HttpRequest) -> Result<Bearer, BearerResponseError> {
            let config: &BearerConfig = req.app_data::<BearerConfig>().ok_or(BearerResponseError::ConfigMissing)?;

            let token_path = match &config.path {
                None => return Ok(Self),
                Some(path) => path,
            };
            let token: String = || -> Result<String, io::Error> {
                let file = &mut fs::File::open(token_path)?;
                read_token(file)
            }()
            .map_err(|_| BearerResponseError::ConfigRead)?;

            let header = match req.headers().get(http::header::AUTHORIZATION) {
                None => return Err(BearerResponseError::MissingAuthHeader),
                Some(header) => header,
            };
            if header.as_bytes() != token.as_bytes() {
                return Err(BearerResponseError::BearerMissmatch);
            }
            Ok(Self)
        }

        ready(from_request_inner(req))
    }
}
