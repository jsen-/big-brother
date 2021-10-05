mod error;
mod kubeconfig;

use crate::k8s_client::api::cluster_config::kubeconfig::Kubeconfig;
pub use error::ClusterConfigError;
use reqwest::{header::HeaderValue, Certificate, Identity};
use std::{
    fs,
    io::{self, BufReader, Read},
};
use ClusterConfigError as Error;

pub enum AuthMethod {
    Identity(Identity),
    Token(reqwest::header::HeaderValue),
}

pub struct ClusterConfig {
    pub server: String,
    pub cacert: Certificate,
    pub auth: AuthMethod,
}

impl ClusterConfig {
    pub fn in_cluster() -> Option<Result<Self, Error>> {
        const TOKEN_PATH: &str = "/var/run/secrets/kubernetes.io/serviceaccount/token";
        const CACERT_PATH: &str = "/var/run/secrets/kubernetes.io/serviceaccount/ca.crt";

        let token = match fs::File::open(TOKEN_PATH) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => return None,
                _ => {
                    return Some(Err(Error::FileOpen {
                        path: TOKEN_PATH.into(),
                        err,
                    }))
                }
            },
        };
        return match in_cluster_inner(token) {
            Ok(cc) => Some(Ok(cc)),
            Err(err) => Some(Err(err)),
        };

        fn read_to_vec(file: fs::File) -> Result<Vec<u8>, io::Error> {
            let mut buf = Vec::new();
            BufReader::new(file).read_to_end(&mut buf)?;
            Ok(buf)
        }
        fn read_token(mut file: fs::File) -> Result<String, io::Error> {
            let mut buf = "Bearer ".to_string();
            file.read_to_string(&mut buf)?;
            Ok(buf)
        }
        fn in_cluster_inner(token_file: fs::File) -> Result<ClusterConfig, Error> {
            let cacert_file = fs::File::open(CACERT_PATH).map_err(|err| Error::FileOpen {
                path: CACERT_PATH.into(),
                err,
            })?;

            let cacert = read_to_vec(cacert_file).map_err(|err| Error::FileRead {
                path: CACERT_PATH.into(),
                err,
            })?;
            let token = read_token(token_file).map_err(|err| Error::FileRead {
                path: TOKEN_PATH.into(),
                err,
            })?;
            Ok(ClusterConfig {
                auth: AuthMethod::Token(HeaderValue::from_str(&token).map_err(|err| Error::InvalidToken(err))?),
                cacert: Certificate::from_pem(&cacert).map_err(|err| Error::Certificate(err))?,
                server: "https://kubernetes.default.svc:443".into(),
            })
        }
    }

    pub fn from_kubeconfig(k: Kubeconfig) -> Result<Self, Error> {
        let contexts = k.contexts;
        let clusters = k.clusters;
        let users = k.users;
        let current_context = k.current_context;
        let context = match contexts.into_iter().find(|c| c.name == current_context) {
            None => Err(()),
            Some(c) => Ok(c),
        }
        .map_err(|_| Error::MissingContext(current_context))?
        .context;
        let current_cluster = context.cluster;
        let current_user = context.user;

        let cluster = match clusters.into_iter().find(|c| c.name == current_cluster) {
            None => Err(()),
            Some(c) => Ok(c),
        }
        .map_err(|_| Error::MissingCluster(current_cluster))?
        .cluster;

        let user = match users.into_iter().find(|c| c.name == current_user) {
            None => Err(()),
            Some(c) => Ok(c),
        }
        .map_err(|_| Error::MissingUser(current_user))?
        .user;

        let client_cert_data =
            base64::decode(user.client_certificate_data).map_err(|err| Error::InvalidBase64Cert(err))?;

        let client_key_data = base64::decode(user.client_key_data).map_err(|err| Error::InvalidBase64Key(err))?;
        let mut pem = client_key_data;
        pem.push(b'\n');
        pem.extend_from_slice(&client_cert_data);
        let identity = Identity::from_pem(&pem).map_err(|err| Error::Identity(err))?;

        let cacert_data =
            base64::decode(cluster.certificate_authority_data).map_err(|err| Error::InvalidBase64Cacert(err))?;
        let cacert = Certificate::from_pem(&cacert_data).map_err(|err| Error::Certificate(err))?;

        Ok(Self {
            auth: AuthMethod::Identity(identity),
            cacert,
            server: cluster.server,
        })
    }

    pub fn detect() -> Result<Self, Error> {
        let cc = match Kubeconfig::from_env() {
            Some(r) => Self::from_kubeconfig(r?)?,
            None => match Self::in_cluster() {
                Some(cc) => cc?,
                None => match Kubeconfig::from_default_path() {
                    Some(r) => Self::from_kubeconfig(r?)?,
                    None => return Err(Error::Detect),
                },
            },
        };
        Ok(cc)
    }
}
