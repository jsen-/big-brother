use reqwest::header::InvalidHeaderValue;
use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ClusterConfigError {
    #[error("Could not open file \"{}\": {:?}", path.display(), err)]
    FileOpen { path: PathBuf, err: io::Error },
    #[error("Could not read file \"{}\": {:?}", path.display(), err)]
    FileRead { path: PathBuf, err: io::Error },
    #[error("Could not deserialize yaml \"{}\": {:?}", path.display(), err)]
    FileDeserialize { path: PathBuf, err: serde_yaml::Error },
    #[error("Could not detect cluster config")]
    Detect,
    #[error("Could not create identity: {:?}", _0)]
    Identity(reqwest::Error),
    #[error("Could not create certificate: {:?}", _0)]
    Certificate(reqwest::Error),
    #[error("Missing context \"{}\"", _0)]
    MissingContext(String),
    #[error("Missing cluster \"{}\"", _0)]
    MissingCluster(String),
    #[error("Missing user \"{}\"", _0)]
    MissingUser(String),
    #[error("Invalid base64 in \"client-certificate-data\": {:?}", _0)]
    InvalidBase64Cert(base64::DecodeError),
    #[error("Invalid base64 in \"client-key-data\": {:?}", _0)]
    InvalidBase64Key(base64::DecodeError),
    #[error("Invalid base64 in \"certificate-authority-data\": {:?}", _0)]
    InvalidBase64Cacert(base64::DecodeError),
    #[error("Invalid token: {:?}", _0)]
    InvalidToken(InvalidHeaderValue),
}
