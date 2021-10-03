pub mod api;

use crate::{engine::ResourceVersion, k8s_client::api::K8sApiError};
use api::{ApiGetter, ApiWatcher};
use backoff::{future::retry_notify, ExponentialBackoff};
use futures::TryFutureExt;
use reqwest::{Certificate, Identity, Method, Request, Response, Url};
use std::{
    str::FromStr,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct K8sClient {
    base_url: Url,
    client: reqwest::Client,
}

#[derive(Debug, thiserror::Error)]
pub enum K8sClientError {
    #[error("Request error {:?}", _0)]
    Reqwest(#[from] reqwest::Error),
    #[error("Url parse error {:?}", _0)]
    UrlParse(#[from] url::ParseError),
    #[error("Url parse error {:?}", _0)]
    K8sApi(#[from] K8sApiError),
}

impl K8sClient {
    pub fn new(base_url: &str, cacert: Certificate, identity: Identity) -> Result<Self, K8sClientError> {
        let client = reqwest::Client::builder()
            .add_root_certificate(cacert)
            .identity(identity)
            .use_rustls_tls() // identity from PEM only works in rustls
            .tls_built_in_root_certs(false)
            .https_only(true)
            .build()
            .map_err(K8sClientError::Reqwest)?;
        let base_url = reqwest::Url::from_str(base_url).map_err(K8sClientError::UrlParse)?;
        Ok(Self { base_url, client })
    }

    fn backoff() -> ExponentialBackoff {
        ExponentialBackoff {
            initial_interval: Duration::ZERO,
            current_interval: Duration::ZERO,
            max_elapsed_time: None,
            randomization_factor: 0.1,
            max_interval: Duration::from_secs(10),
            multiplier: 1.3,
            clock: Default::default(),
            start_time: Instant::now(),
        }
    }
    fn notify(err: K8sClientError, duration: Duration) {
        eprintln!(
            "Error sending request (duration: {}.{:.3}):\n{:?}",
            duration.as_secs(),
            duration.as_millis(),
            err
        );
    }
    async fn send(&self, method: &Method, uri: &str, body: Vec<u8>) -> Result<Response, K8sClientError> {
        let url = &self.base_url.join(uri)?;
        let send = move || {
            let mut req = Request::new(method.clone(), url.clone());
            *req.body_mut() = Some(reqwest::Body::from(body.clone()));

            self.client.execute(req).map_err(|e| {
                if e.is_connect() || e.is_decode() || e.is_timeout() {
                    backoff::Error::Transient(K8sClientError::from(e))
                } else {
                    backoff::Error::Permanent(K8sClientError::from(e))
                }
            })
        };
        retry_notify(Self::backoff(), &send, Self::notify).await
    }

    pub async fn get<T: ApiGetter>(&self, getter: &T) -> Result<T::Output, K8sClientError> {
        let req = getter.get();
        let resp = self.send(&req.method, &req.relative_url, req.body).await?;
        let status = resp.status();
        if !(req.status_check)(status) {
            return Err(K8sClientError::K8sApi(K8sApiError::UnexpectedStatus(status)));
        }
        let bytes = resp.bytes().await?;
        let result = (req.response)(&bytes)?;
        Ok(result)
    }

    pub async fn watch<T: ApiWatcher>(&self, watcher: &T, rv: ResourceVersion) -> Result<Response, K8sClientError> {
        let req = watcher.watch(rv);
        println!("{:?}", req);
        self.send(&req.method, &req.relative_url, req.body).await
    }
}
