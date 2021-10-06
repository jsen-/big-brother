mod engine;
mod error;
mod event;
mod k8s_client;
mod utils;

use actix_web::{
    body::BodyStream,
    web::{self, Bytes, Data},
    App, HttpResponse, HttpServer, Responder,
};
use engine::Cache;
use error::Error;
use event::EventType;
use k8s_client::{
    api::{cluster_config::ClusterConfig, ResourceVersion},
    K8sClient,
};
use serde::Deserialize;
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

#[derive(serde::Serialize)]
struct OutputEvent {
    #[serde(rename = "type")]
    event_type: EventType,
    object: destream_json::Value,
}

#[derive(Debug, Clone)]
struct AppData {
    cache: Arc<RwLock<Cache>>,
}

fn main() -> Result<(), Error> {
    let cc = ClusterConfig::detect()?;
    let k8s_client = K8sClient::from_cluster_config(cc)?;
    actix_web::rt::System::new().block_on(async move {
        let engine = engine::watch(k8s_client).await?;
        let cache = engine.cache().clone();
        let server = HttpServer::new(move || {
            App::new() //
                .app_data(Data::new(AppData { cache: cache.clone() }))
                .service(watch)
                .service(list)
                .service(status)
        })
        .bind(("0.0.0.0", 8080))
        .map_err(Error::ServerBind)?;
        server.run().await.map_err(Error::ServerRun)?;
        Ok::<_, Error>(())
    })?;
    Ok(())
}

#[derive(Debug, Deserialize)]
enum Filter {
    #[serde(rename = "include")]
    Include(String),
    #[serde(rename = "exclude")]
    Exclude(String),
}

#[derive(Debug, Deserialize)]
struct Query {
    #[serde(rename = "resourceVersion")]
    resource_version: Option<ResourceVersion>,
    #[serde(flatten)]
    filter: Option<Filter>,
}

#[actix_web::get("/watch")]
async fn watch(query: web::Query<Query>, appdata: web::Data<AppData>) -> impl Responder {
    let filter: Box<dyn Fn(&str) -> bool> = match &query.filter {
        None => Box::new(move |_| true),
        Some(Filter::Include(filter)) => {
            let names = filter.split(',').map(|x| x.to_string()).collect::<HashSet<String>>();
            Box::new(move |name| names.contains(name))
        }
        Some(Filter::Exclude(filter)) => {
            let names = filter.split(',').map(|x| x.to_string()).collect::<HashSet<_>>();
            Box::new(move |name| !names.contains(name))
        }
    };
    let cache = appdata.get_ref().cache.read().await;
    let stream = cache.stream(query.resource_version);
    let stream = stream.filter_map(move |evt| {
        let (res, evt) = otry!(evt);
        if filter(&res.kind) {
            let mut vec = otry!(serde_json::to_vec(&evt));
            vec.push(b'\n');
            Some(Ok::<_, Error>(Bytes::from(vec)))
        } else {
            None
        }
    });
    let ret = BodyStream::new(stream);
    HttpResponse::Ok().body(ret)
}

#[actix_web::get("/list")]
async fn list(appdata: web::Data<AppData>) -> impl Responder {
    let cache = appdata.get_ref().cache.read().await;
    HttpResponse::Ok().body(cache.list())
}

#[actix_web::get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok()
}
