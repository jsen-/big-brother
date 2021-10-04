use crate::k8s_client::api::{ResourceId, ResourceVersion};
use destream_json::Value;
use serde::Serialize;
use std::fmt;

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

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub resource: ResourceId,
    pub value: Value,
    pub resource_version: ResourceVersion,
}

#[derive(Debug, thiserror::Error)]
pub enum EventParseError {
    #[error("\"resourceVersion\" is not type string: {:?}", _0)]
    InvalidResourceVersion(String),
    #[error("\"namespace\" is not type string: {:?}", _0)]
    NamespaceNotString(Value),
    #[error("Missing \"name\" or \"resourceVersion\" property")]
    MissingNameOrResourceVersion,
    #[error("Missing \"apiVersion\", \"kind\" or \"metadata\" property")]
    MissingApiVersionKindMetadata,
    #[error("\"object\" is not type object: {:?}", _0)]
    ObjectNotObject(Value),
    #[error("Missing \"object\"")]
    MissingObject,
    #[error("\"type\" is not a known value: {:?}", _0)]
    UnknownEvent(String),
    #[error("\"type\" is not string: {:?}", _0)]
    TypeNotString(Value),
    #[error("Missing \"type\"")]
    MissingType,
    #[error("Event is not type object: {:?}", _0)]
    RootNotObject(Value),
}

impl TryFrom<Value> for Event {
    type Error = EventParseError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(ref map) => {
                let (k8s_resource, resource_version) = match map.get("object") {
                    Some(Value::Map(obj)) => {
                        match (obj.get("apiVersion"), obj.get("kind"), obj.get("metadata")) {
                            (
                                Some(Value::String(api_version)),
                                Some(Value::String(kind)),
                                Some(Value::Map(metadata)),
                            ) => {
                                match (metadata.get("name"), metadata.get("resourceVersion")) {
                                    (Some(Value::String(name)), Some(Value::String(resource_version_str))) => {
                                        let rv = match resource_version_str.parse::<ResourceVersion>() {
                                            Ok(rv) => rv,
                                            Err(e) => {
                                                eprintln!("Invalid resource version received: {:?}", e);
                                                return Err(EventParseError::InvalidResourceVersion(
                                                    resource_version_str.clone(),
                                                ));
                                                // invalid resourceVersion
                                            }
                                        };
                                        let ns = match metadata.get("namespace") {
                                            Some(Value::String(namespace)) => Some(namespace),
                                            Some(x) => return Err(EventParseError::NamespaceNotString(x.clone())),
                                            None => None,
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
                                    _ => return Err(EventParseError::MissingNameOrResourceVersion), // missing name or resource_version
                                }
                            }
                            _ => return Err(EventParseError::MissingApiVersionKindMetadata), // missing api_version, kind or metadata
                        }
                    }
                    Some(obj) => return Err(EventParseError::ObjectNotObject(obj.clone())), // object is not of type object
                    None => return Err(EventParseError::MissingObject), // object is not of type object
                };

                let event_type = match map.get("type") {
                    Some(Value::String(ty)) => {
                        if ty.as_str() == "ADDED" {
                            EventType::Added
                        } else if ty.as_str() == "MODIFIED" {
                            EventType::Modified
                        } else if ty.as_str() == "DELETED" {
                            EventType::Deleted
                        } else {
                            // unknown event type, e.g. BOOKMARK
                            return Err(EventParseError::UnknownEvent(ty.clone()));
                        }
                    }
                    Some(val) => return Err(EventParseError::TypeNotString(val.clone())),
                    None => return Err(EventParseError::MissingType),
                };
                Ok(Event {
                    event_type,
                    resource: k8s_resource,
                    value,
                    resource_version: resource_version.try_into().unwrap(),
                })
            }
            val @ _ => return Err(EventParseError::RootNotObject(val)),
        }
    }
}
