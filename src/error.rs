use crate::{
    event::EventParseError,
    k8s_client::{api::cluster_config::ClusterConfigError, K8sClientError},
};
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
    DeserializeStream(#[from] destream_json::de::Error),
    #[error("Stream deserialization error: {:?}", _0)]
    Deserialize(#[from] serde_json::Error),
    #[error("Error parsing kubernetes event: {:?}", _0)]
    EventParseError(#[from] EventParseError),
    #[error("Server could not bind to port: {:?}", _0)]
    ServerBind(io::Error),
    #[error("Server could not run server: {:?}", _0)]
    ServerRun(io::Error),
    #[error("Could not obtain cluster config: {:?}", _0)]
    ClusterConfig(#[from] ClusterConfigError),
    #[error("Unable to receive value from stream: {:?}", _0)]
    StreamRecv(#[from] tokio_stream::wrappers::errors::BroadcastStreamRecvError),
}
