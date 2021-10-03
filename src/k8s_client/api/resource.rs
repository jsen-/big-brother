use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct ListMeta {
    #[serde(rename = "resourceVersion")]
    pub resource_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResourceList {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: ListMeta,
    pub items: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListItem {
    pub metadata: Metadata,
}

/// allows us to add `api_version` and `kind` fields to `ListItem` objects
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resource {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    // pub metadata: Metadata,
    #[serde(flatten)]
    pub rest: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub name: String,
    pub namespace: Option<String>,
    #[serde(rename = "resourceVersion")]
    pub resource_version: String,
}
