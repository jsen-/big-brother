use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiResource {
    pub categories: Option<Vec<String>>,
    pub group: Option<String>,
    pub kind: String,
    pub name: String,
    pub namespaced: bool,
    #[serde(rename = "shortNames")]
    pub short_names: Option<Vec<String>>,
    #[serde(rename = "singularName")]
    pub singular_name: String,
    #[serde(rename = "storageVersionHash")]
    pub storage_version_hash: Option<String>,
    pub verbs: Vec<String>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiResourceList {
    #[serde(rename = "groupVersion")]
    pub group_version: String,
    pub resources: Vec<ApiResource>,
}
