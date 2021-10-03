use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiGroupVersion {
    #[serde(rename = "groupVersion")]
    pub group_version: String,
    pub version: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiGroupListItem {
    pub name: String,
    pub versions: Vec<ApiGroupVersion>,
    #[serde(rename = "preferredVersion")]
    pub preferred_version: Option<ApiGroupVersion>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiGroupList {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub groups: Vec<ApiGroupListItem>,
}
