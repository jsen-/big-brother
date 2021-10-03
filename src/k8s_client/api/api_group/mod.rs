mod api_group_list;

pub use api_group_list::ApiGroupList;
use api_group_list::ApiGroupVersion;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ApiGroup {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub versions: Vec<ApiGroupVersion>,
    #[serde(rename = "preferredVersion")]
    pub preferred_version: ApiGroupVersion,
}
