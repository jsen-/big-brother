use crate::k8s_client::api::{ResourceId, ResourceVersion};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use tokio::sync::broadcast;
use tokio_stream::{
    wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    Stream, StreamExt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OutputEventType {
    Modified,
    Deleted,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct OutputEvent {
    #[serde(rename = "type")]
    ty: OutputEventType,
    object: Value,
}

#[derive(Debug)]
pub struct Cache {
    resources: HashMap<ResourceId, Option<(ResourceVersion, Value)>>,
    changes: BTreeMap<ResourceVersion, ResourceId>,
    tx: broadcast::Sender<(ResourceId, OutputEvent)>,
}

fn deleted_event(res: ResourceId, rv: ResourceVersion) -> Value {
    let mut meta = [
        ("name".to_string(), Value::String(res.name)),
        ("resourceVersion".to_string(), Value::String(rv.to_string())),
    ]
    .into_iter()
    .collect::<serde_json::value::Map<_, _>>();
    if let Some(ns) = res.namespace {
        meta.insert("namespace".to_string(), Value::String(ns));
    }

    let obj = [
        ("apiVersion".to_string(), Value::String(res.api_version)),
        ("kind".to_string(), Value::String(res.kind)),
        ("metadata".to_string(), Value::Object(meta)),
    ]
    .into_iter()
    .collect::<serde_json::value::Map<_, _>>();
    serde_json::Value::Object(obj)
}

impl Cache {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Cache {
            resources: HashMap::new(),
            changes: BTreeMap::new(),
            tx,
        }
    }
    fn update_internal(&mut self, res: ResourceId, rv: ResourceVersion, value: Option<Value>) {
        if let Some(Some((old_rv, _))) = self.resources.insert(res.clone(), value.map(|v| (rv, v))) {
            self.changes.remove(&old_rv);
        }
        self.changes.insert(rv, res);
    }

    pub fn update(&mut self, res: ResourceId, rv: ResourceVersion, value: Value) {
        self.update_internal(res.clone(), rv, Some(value.clone()));
        self.tx
            .send((
                res,
                OutputEvent {
                    ty: OutputEventType::Modified,
                    object: value,
                },
            ))
            .ok(); // `send` will fail when there are no currently receivers, but we don't really care
    }

    pub fn remove(&mut self, res: ResourceId, rv: ResourceVersion) {
        self.update_internal(res.clone(), rv, None);
        self.tx
            .send((
                res.clone(),
                OutputEvent {
                    ty: OutputEventType::Deleted,
                    object: deleted_event(res, rv),
                },
            ))
            .ok(); // `send` will fail when there are no currently receivers, but we don't really care
    }
    pub fn list(&self) -> String {
        let it = self.resources.iter().map(|(res, v)| match v {
            Some((rv, _)) => format!(
                "<td>{}</td><td>{}</td><td>{:?}</td><td>{}</td><td>{}</td>",
                res.api_version, res.kind, res.namespace, res.name, rv
            ),
            None => format!(
                "<td>{}</td><td>{}</td><td>{:?}</td><td>{}</td><td></td>",
                res.api_version, res.kind, res.namespace, res.name
            ),
        });
        let head = std::iter::once("<table><tr><th>apiVersion</th><th>kind</th><th>(namespace)</th><th>name</th><th>resourceVersion</th></tr><tr>".to_string());
        let it = it.intersperse("</tr><tr>".to_string());
        let tail = std::iter::once("</tr></table>".to_string());
        let s = head.chain(it).chain(tail).collect::<String>();
        s
    }
    pub fn stream(
        &self,
        rv: Option<ResourceVersion>,
    ) -> impl Stream<Item = Result<(ResourceId, OutputEvent), BroadcastStreamRecvError>> {
        let range = match rv {
            Some(rv) => self.changes.range(rv..),
            None => self.changes.range(..),
        };
        let changes = range
            .filter_map(|(_rv, res)| {
                self.resources[res].as_ref().map(|(_rv, value)| {
                    Ok((
                        res.clone(),
                        OutputEvent {
                            ty: OutputEventType::Modified,
                            object: value.clone(),
                        },
                    ))
                })
            })
            .collect::<Vec<_>>();
        // TODO: prove that we can't skip/duplicate events here
        let event_stream = BroadcastStream::new(self.tx.subscribe());
        let stream = tokio_stream::iter(changes);
        stream.chain(event_stream)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_res(api_version: &str, kind: &str, name: &str, namespace: Option<&str>) -> ResourceId {
        ResourceId {
            api_version: api_version.into(),
            kind: kind.into(),
            name: name.into(),
            namespace: namespace.map(Into::into),
        }
    }
    fn make_evt_modified(object: Value) -> OutputEvent {
        OutputEvent {
            object,
            ty: OutputEventType::Modified,
        }
    }
    fn make_evt_deleted(res: ResourceId, rv: ResourceVersion) -> OutputEvent {
        let object = deleted_event(res, rv);
        OutputEvent {
            object,
            ty: OutputEventType::Deleted,
        }
    }

    #[tokio::test]
    async fn changes_add() {
        let mut cache = Cache::new();
        let res1 = make_res("av", "k", "n1", None);
        let res2 = make_res("av", "k", "n2", None);
        cache.update(res1.clone(), 1, Value::Null);
        cache.update(res2.clone(), 2, Value::Null);
        let mut stream = Box::pin(cache.stream(None));
        drop(cache);
        assert_eq!(stream.next().await, Some(Ok((res1, make_evt_modified(Value::Null)))));
        assert_eq!(stream.next().await, Some(Ok((res2, make_evt_modified(Value::Null)))));
        assert_eq!(stream.next().await, None);
    }
    #[tokio::test]
    async fn changes_overwrite() {
        let mut cache = Cache::new();
        let res = make_res("av", "k", "n", None);
        cache.update(res.clone(), 1, Value::Null);
        cache.update(res.clone(), 2, Value::Null);
        let mut stream = Box::pin(cache.stream(None));
        drop(cache);
        assert_eq!(stream.next().await, Some(Ok((res, make_evt_modified(Value::Null)))));
        assert_eq!(stream.next().await, None);
    }
    #[tokio::test]
    async fn del_before_listening() {
        let res = make_res("av", "k", "n", None);
        let mut cache = Cache::new();
        cache.update(res.clone(), 1, Value::Null);
        cache.remove(res.clone(), 2);
        let mut stream = Box::pin(cache.stream(None));
        drop(cache);
        assert_eq!(stream.next().await, None);
    }
    #[tokio::test]
    async fn del_after_listening() {
        let res = make_res("av", "k", "n", None);
        let mut cache = Cache::new();
        cache.update(res.clone(), 1, Value::Null);
        let mut stream = Box::pin(cache.stream(None));
        cache.remove(res.clone(), 2);
        drop(cache);
        assert_eq!(
            stream.next().await,
            Some(Ok((res.clone(), make_evt_modified(Value::Null))))
        );
        assert_eq!(stream.next().await, Some(Ok((res.clone(), make_evt_deleted(res, 2)))));
        assert_eq!(stream.next().await, None);
    }
}
