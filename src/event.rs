use crate::{engine::ResourceVersion, error::Error};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Hash)]
pub struct ResourceId {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EventType {
    Added,
    Modified,
    Deleted,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Added => write!(f, "added"),
            EventType::Modified => write!(f, "modified"),
            EventType::Deleted => write!(f, "deleted"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub event_type: EventType,
    pub resource: ResourceId,
    pub value: destream_json::Value,
    pub resource_version: ResourceVersion,
}

// impl<'en> destream::ToStream<'en> for Event {
//     fn to_stream<E: destream::Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
//         let mut map = encoder.encode_map(Some(2))?;
//         let evt_type = match self.event_type {
//             EventType::Deleted => "DELETED",
//             EventType::Modified => "MODIFIED",
//             EventType::Added => "ADDED",
//         };
//         map.encode_entry("type", evt_type)?;
//         map.encode_entry("object", &self.value)?;
//         map.end()
//     }
// }

impl TryFrom<destream_json::Value> for Event {
    type Error = Error;
    fn try_from(value: destream_json::Value) -> Result<Self, Self::Error> {
        match value {
            destream_json::Value::Map(ref map) => {
                let (k8s_resource, resource_version) = match map.get("object") {
                    Some(destream_json::Value::Map(obj)) => {
                        match (obj.get("apiVersion"), obj.get("kind"), obj.get("metadata")) {
                            (
                                Some(destream_json::Value::String(api_version)),
                                Some(destream_json::Value::String(kind)),
                                Some(destream_json::Value::Map(metadata)),
                            ) => {
                                match (metadata.get("name"), metadata.get("resourceVersion")) {
                                    (
                                        Some(destream_json::Value::String(name)),
                                        Some(destream_json::Value::String(resource_version_str)),
                                    ) => {
                                        let rv = match resource_version_str.parse::<ResourceVersion>() {
                                            Ok(rv) => rv,
                                            Err(e) => {
                                                eprintln!("Invalid resource version received: {:?}", e);
                                                return Err(Error::Skipped); // invalid resourceVersion
                                            }
                                        };
                                        let ns = match metadata.get("namespace") {
                                            Some(destream_json::Value::String(namespace)) => Some(namespace),
                                            None => None,
                                            _ => return Err(Error::Skipped), // invalid namespace
                                        };
                                        (
                                            ResourceId {
                                                api_version: api_version.clone(),
                                                kind: kind.clone(),
                                                name: name.clone(),
                                                namespace: ns.cloned(),
                                            },
                                            rv,
                                        )
                                    }
                                    _ => return Err(Error::Skipped), // missing name or resource_version
                                }
                            }
                            _ => return Err(Error::Skipped), // missing api_version, kind or metadata
                        }
                    }
                    _ => return Err(Error::Skipped), // object is not of type object
                };

                let event_type = if let Some(destream_json::Value::String(ty)) = map.get("type") {
                    if ty.as_str() == "ADDED" {
                        EventType::Added
                    } else if ty.as_str() == "MODIFIED" {
                        EventType::Modified
                    } else if ty.as_str() == "DELETED" {
                        EventType::Deleted
                    } else {
                        return Err(Error::Skipped); // unknown event type, e.g. BOOKMARK
                    }
                } else {
                    return Err(Error::Skipped); // event type is missing or not string
                };
                Ok(Event {
                    event_type,
                    resource: k8s_resource,
                    value,
                    resource_version: resource_version.try_into().unwrap(),
                })
            }
            _ => return Err(Error::Skipped),
        }
    }
}
