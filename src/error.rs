use crate::{event::EventParseError, k8s_client::K8sClientError};
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes client error: {:?}", _0)]
    K8sClient(#[from] K8sClientError),
    #[error("URL parse error: {:?}", _0)]
    UrlParse(#[from] url::ParseError),
    #[error("Request error: {:?}", _0)]
    Transport(#[from] reqwest::Error),
    #[error("Stream deserialization error: {:?}", _0)]
    Deserialize(#[from] destream_json::de::Error),
    #[error("Error parsing kubernetes event: {:?}", _0)]
    EventParseError(#[from] EventParseError),
    #[error("Server could not bind to port: {:?}", _0)]
    ServerBind(io::Error),
    #[error("Server could not run server: {:?}", _0)]
    ServerRun(io::Error),
}
