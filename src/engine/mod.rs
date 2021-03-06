mod cache;
mod k8s_resource_output;
mod to_serde;

use crate::{
    engine::to_serde::convert_value_to_value,
    error::Error,
    event::{Event, EventType},
    k8s_client::{
        api::{
            ApiGroupListGetter, ApiResource, ApiResourceListGetter, ApiVersionListGetter, CoreResourceListGetter,
            ListItem, Resource, ResourceId, ResourceListGetter, ResourceVersion,
        },
        K8sClient,
    },
};
pub use cache::Cache;
use destream_json::{try_decode_iter, Value as DValue};
use std::{convert::TryFrom, sync::Arc};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

pub async fn watch(k8s_client: K8sClient) -> Result<Engine, Error> {
    let engine = Engine {
        k8s_client,
        cache: Arc::new(RwLock::new(Cache::new())),
    };
    engine.watch().await?;
    Ok(engine)
}

#[derive(Debug)]
pub struct Engine {
    k8s_client: K8sClient,
    cache: Arc<RwLock<Cache>>,
}

impl Engine {
    fn watch_resource(&self, group: Option<String>, version: String, api_resource: ApiResource) {
        // this filters out all "*/status", "*/scale", "*/approval", "v1/bindings" and "v1/componentstatuses" which we don't care about
        if !api_resource.verbs.contains(&"watch".to_string()) {
            return;
        }
        println!("watching \"{}\"", api_resource.name);
        let k8s_client = self.k8s_client.clone();
        let cache = Arc::clone(&self.cache);
        tokio::task::spawn(async move {
            let getter = ResourceListGetter {
                group: group.clone(),
                version: version.clone(),
                plural: api_resource.name.clone(),
            };
            let api_version = match &group {
                None => version.clone(),
                Some(group) => format!("{}/{}", group, version),
            };
            match k8s_client.get(&getter).await {
                Err(err) => eprintln!("k8s client error {:?}", err),
                Ok(resource_list) => match resource_list.metadata.resource_version.parse::<ResourceVersion>() {
                    Err(_) => {
                        eprintln!("Invalid resource version {:?}", resource_list.metadata.resource_version);
                    }
                    Ok(rv) => {
                        let mut last_rv = rv;
                        {
                            let mut writer = cache.write().await;
                            for resource in resource_list.items {
                                match serde_json::from_value::<ListItem>(resource.clone()) {
                                    Err(err) => {
                                        eprintln!("{:?} {} {}", group, version, api_resource.name);
                                        eprintln!("{:#?}", resource);
                                        eprintln!("Could not deserialize value to resource: {:?}", err)
                                    }
                                    Ok(res) => match res.metadata.resource_version.parse::<ResourceVersion>() {
                                        Ok(rv) => {
                                            let k8s_resource = ResourceId {
                                                api_version: api_version.clone(),
                                                kind: api_resource.kind.clone(),
                                                name: res.metadata.name.clone(),
                                                namespace: res.metadata.namespace.clone(),
                                            };
                                            let value = Resource {
                                                api_version: resource_list.api_version.clone(),
                                                kind: resource_list.kind.clone(),
                                                rest: resource,
                                            };
                                            // expectations:
                                            // serde_json::to_value fails if `T`'s implementation of `Serialize` decides to fail, or if `T` contains a map with non-string keys.
                                            // None of those cases shall happen
                                            writer.update(
                                                k8s_resource,
                                                rv,
                                                serde_json::to_value(&value).expect("Resource serialization failed"),
                                            );
                                        }
                                        Err(_) => {
                                            eprintln!("Invalid resourceVersion {:?}", res.metadata.resource_version)
                                        }
                                    },
                                }
                            }
                        } // drop writer

                        loop {
                            match k8s_client.watch(&getter, last_rv).await {
                                Err(err) => eprintln!("Watch error: {:?}", err),
                                Ok(response) => {
                                    let json_stream =
                                        try_decode_iter::<_, _, DValue>((), response.bytes_stream()).await;
                                    tokio::pin!(json_stream);
                                    while let Some(value) = json_stream.next().await {
                                        match value
                                            .map_err(Error::DeserializeStream)
                                            .and_then(|value| Event::try_from(value).map_err(Error::EventParseError))
                                        {
                                            Err(e) => eprintln!("{:?}", e),
                                            Ok(evt) => {
                                                last_rv = evt.resource_version;
                                                let mut writer = cache.write().await;
                                                match &evt.event_type {
                                                    EventType::Added | EventType::Modified => writer.update(
                                                        evt.resource.clone(),
                                                        evt.resource_version,
                                                        convert_value_to_value(&evt.value),
                                                    ),
                                                    EventType::Deleted => {
                                                        writer.remove(evt.resource.clone(), evt.resource_version)
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
        });
    }

    async fn watch(&self) -> Result<(), Error> {
        let group_list = self.k8s_client.get(&ApiGroupListGetter).await?;
        let api_versions = group_list.groups.into_iter().filter_map(|api_group| {
            let name = api_group.name;
            api_group.preferred_version.map(|pref| (name, pref.version))
        });
        // TODO: perform some validation of group and version, so url can be safely
        // constructed from `format!("/apis/{}/{}/", group, version)`
        // currently, this can cause panic in ApiWatcher
        // however, I think it's currently impossible to construct k8s resource, that could trigger this panic
        // this also applies to all `ApiGetters` below
        for (group, version) in api_versions {
            let api_resources = self
                .k8s_client
                .get(&ApiResourceListGetter {
                    group: Some(&group),
                    version: &version,
                })
                .await?;
            for resource in api_resources.resources {
                self.watch_resource(Some(group.clone()), version.clone(), resource.clone());
            }
        }

        let api_versions = self.k8s_client.get(&ApiVersionListGetter).await?;
        for version in api_versions.versions {
            let core_api_resources = self
                .k8s_client
                .get(&CoreResourceListGetter {
                    version: version.clone(),
                })
                .await?;
            for core_resource in core_api_resources.resources {
                self.watch_resource(None, version.clone(), core_resource.clone());
            }
        }
        Ok(())
    }
    pub fn cache(&self) -> &Arc<RwLock<Cache>> {
        &self.cache
    }
}
