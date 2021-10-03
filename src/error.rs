use std::io;
use crate::k8s_client::K8sClientError;

#[derive(Debug)]
pub enum Error {
    K8sClient(K8sClientError),
    UrlParse(url::ParseError),
    Response(ResponseError),
    Transport(reqwest::Error),
    Deserialize(destream_json::de::Error),
    Skipped,
    Io(io::Error),
}

impl From<K8sClientError> for Error {
    fn from(src: K8sClientError) -> Self {
        Self::K8sClient(src)
    }
}
impl From<io::Error> for Error {
    fn from(src: io::Error) -> Self {
        Self::Io(src)
    }
}
impl From<ResponseError> for Error {
    fn from(src: ResponseError) -> Self {
        Self::Response(src)
    }
}
impl From<url::ParseError> for Error {
    fn from(src: url::ParseError) -> Self {
        Self::UrlParse(src)
    }
}
impl From<reqwest::Error> for Error {
    fn from(src: reqwest::Error) -> Self {
        Self::Transport(src)
    }
}
impl From<destream_json::de::Error> for Error {
    fn from(src: destream_json::de::Error) -> Self {
        Self::Deserialize(src)
    }
}

#[derive(Debug)]
pub enum ResponseError {
    Deserialize(serde_json::Error),
    Transfer(reqwest::Error),
    Io(io::Error),
}

impl From<reqwest::Error> for ResponseError {
    fn from(src: reqwest::Error) -> Self {
        Self::Transfer(src)
    }
}
impl From<io::Error> for ResponseError {
    fn from(src: io::Error) -> Self {
        Self::Io(src)
    }
}
impl From<serde_json::Error> for ResponseError {
    fn from(src: serde_json::Error) -> Self {
        Self::Deserialize(src)
    }
}
