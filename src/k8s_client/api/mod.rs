mod api_group;
mod api_resource;
mod api_version;
mod resource;

use api_group::{ApiGroup, ApiGroupList};
pub use api_resource::{ApiResource, ApiResourceList};
use reqwest::{Method, StatusCode};
pub use resource::{ListItem, Metadata, Resource, ResourceList};
use std::fmt;
use uriparse::relative_reference::RelativeReference;

use crate::engine::ResourceVersion;

use self::api_version::ApiVersions;

pub trait ApiWatcher: ApiGetter {
    fn watch(&self, rv: ResourceVersion) -> Req<Self::Output> {
        let mut req = self.get();
        let mut uri = RelativeReference::try_from(req.relative_url.as_str())
            .expect(&format!("Invalid uri reference {:?}", req.relative_url));
        let query = match uri.query() {
            Some(q) => qstring::QString::from(q.as_str())
                .into_iter()
                .filter_map(|(key, val)| {
                    if key == "watch" || key == "resourceVersion" {
                        None
                    } else {
                        Some((key, val))
                    }
                })
                .chain(std::iter::once(("watch".into(), "true".into())))
                .map(|(key, value)| format!("{}={}", key, value))
                .intersperse("&".to_string())
                .collect::<String>(),
            None => format!("watch=true&resourceVersion={}", rv),
        };
        uri.set_query(Some(query.as_str())).unwrap();
        req.relative_url = uri.to_string();
        req
    }
}

pub trait ApiGetter: Clone {
    type Output;
    fn get(&self) -> Req<Self::Output>;
}

#[derive(Debug, thiserror::Error)]
pub enum K8sApiError {
    #[error("Unexpected status [{}]", _0)]
    UnexpectedStatus(StatusCode),
    #[error("Deserialization error: {:?}", _0)]
    Deserialize(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct Req<T> {
    pub method: Method,
    pub relative_url: String,
    pub body: Vec<u8>,
    pub status_check: fn(StatusCode) -> bool,
    pub response: fn(&[u8]) -> Result<T, K8sApiError>,
}

impl<T> fmt::Debug for Req<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Req")
            .field("method", &self.method)
            .field("relative_url", &self.relative_url)
            .field("response", &"fn(...)")
            .finish()
    }
}

impl<T> Req<T> {
    fn get<S: Into<String>>(relative_url: S, f: fn(&[u8]) -> Result<T, K8sApiError>) -> Self {
        Self {
            method: Method::GET,
            relative_url: relative_url.into(),
            body: Vec::new(),
            status_check: |status_code| status_code == StatusCode::OK,
            response: f,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiResourceListGetter<'a> {
    pub group: Option<&'a str>,
    pub version: &'a str,
}
impl<'a> ApiGetter for ApiResourceListGetter<'a> {
    type Output = ApiResourceList;
    fn get(&self) -> Req<Self::Output> {
        let path = match self.group {
            Some(group) => format!("/apis/{}/{}/", group, self.version),
            None => format!("/api/{}", self.version),
        };

        Req::get(path, |resp| Ok(serde_json::from_slice(resp)?))
    }
}

#[derive(Debug, Clone)]
pub struct ApiGroupListGetter;
impl ApiGetter for ApiGroupListGetter {
    type Output = ApiGroupList;
    fn get(&self) -> Req<Self::Output> {
        Req::get("/apis", |resp| Ok(serde_json::from_slice(resp)?))
    }
}

#[derive(Debug, Clone)]
pub struct ApiVersionListGetter;
impl ApiGetter for ApiVersionListGetter {
    type Output = ApiVersions;
    fn get(&self) -> Req<Self::Output> {
        Req::get("/api", |resp| Ok(serde_json::from_slice(resp)?))
    }
}

#[derive(Debug, Clone)]
pub struct CoreResourceListGetter {
    pub version: String,
}
impl ApiGetter for CoreResourceListGetter {
    type Output = ApiResourceList;
    fn get(&self) -> Req<Self::Output> {
        Req::get(format!("/api/{}", self.version), |resp| {
            Ok(serde_json::from_slice(resp)?)
        })
    }
}

#[derive(Debug, Clone)]
pub struct ApiGroupGetter<'a> {
    group_name: &'a str,
}
impl<'a> ApiGetter for ApiGroupGetter<'a> {
    type Output = ApiGroup;
    fn get(&self) -> Req<Self::Output> {
        Req::get(format!("/apis/{}", self.group_name), |resp| {
            Ok(serde_json::from_slice(resp)?)
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResourceListGetter {
    pub group: Option<String>,
    pub version: String,
    pub plural: String,
}

impl ApiGetter for ResourceListGetter {
    type Output = ResourceList;
    fn get(&self) -> Req<Self::Output> {
        let path = match self.group {
            Some(ref group) => format!("/apis/{}/{}/{}", group, self.version, self.plural),
            None => format!("/api/{}/{}", self.version, self.plural),
        };
        Req::get(path, |resp| Ok(serde_json::from_slice(resp)?))
    }
}
impl ApiWatcher for ResourceListGetter {}
