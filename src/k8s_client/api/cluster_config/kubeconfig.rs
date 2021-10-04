use super::error::ClusterConfigError;
use serde::Deserialize;
use std::{fs, io, path::Path};

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Kubeconfig {
    pub clusters: Vec<NamedCluster>,
    pub users: Vec<NamedUser>,
    pub contexts: Vec<NamedContext>,
    #[serde(rename = "current-context")]
    pub current_context: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NamedCluster {
    pub name: String,
    pub cluster: Cluster,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Cluster {
    pub server: String,
    #[serde(rename = "certificate-authority-data")]
    pub certificate_authority_data: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NamedUser {
    pub name: String,
    pub user: User,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct User {
    #[serde(rename = "client-certificate-data")]
    pub client_certificate_data: String,
    #[serde(rename = "client-key-data")]
    pub client_key_data: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NamedContext {
    pub name: String,
    pub context: Context,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Context {
    pub cluster: String,
    pub user: String,
}

impl Kubeconfig {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Kubeconfig, ClusterConfigError> {
        let file = fs::File::open(path.as_ref()).map_err(|err| ClusterConfigError::FileOpen {
            path: path.as_ref().into(),
            err,
        })?;
        let kubeconfig: Kubeconfig =
            serde_yaml::from_reader(file).map_err(|err| ClusterConfigError::FileDeserialize {
                path: path.as_ref().into(),
                err,
            })?;
        Ok(kubeconfig)
    }

    pub fn from_default_path() -> Option<Result<Kubeconfig, ClusterConfigError>> {
        let homedir = dirs::home_dir()?;
        let path = homedir.join(".kube").join("config");
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => return None,
                _ => return Some(Err(ClusterConfigError::FileOpen { path, err })),
            },
        };
        Some(serde_yaml::from_reader(file).map_err(|err| ClusterConfigError::FileDeserialize { path, err }))
    }

    pub fn from_env() -> Option<Result<Self, ClusterConfigError>> {
        match std::env::var_os("KUBECONFIG") {
            Some(path) => Some(Self::from_path(path)),
            None => None,
        }
    }
}
