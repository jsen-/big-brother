#![feature(more_qualified_paths)]
#![feature(const_option)]
mod error;
mod k8s_client;
mod engine;
mod event;

use std::sync::Arc;

use actix_web::{
    body::BodyStream,
    web::{self, Bytes, Data},
    App, HttpResponse, HttpServer, Responder,
};
use k8s_client::K8sClient;
use engine::{watch, Cache, ResourceVersion};
use error::Error;
use event::EventType;
use futures::StreamExt;
use reqwest::{Certificate, Identity};
use tokio::sync::RwLock;

static PEM: &[u8] = include_bytes!("../identity.pem");
static CACERT: &[u8] = include_bytes!("../root.crt");

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
    let cacert = Certificate::from_pem(CACERT).unwrap();
    let identity = Identity::from_pem(PEM).unwrap();
    let base_url = "https://localhost:6443";

    let k8s_client = K8sClient::new(base_url, cacert, identity)?;

    actix_web::rt::System::new().block_on(async move {
        let engine = watch(k8s_client).await?;
        let cache = engine.cache().clone();
        let server = HttpServer::new(move || {
            App::new() //
                .app_data(Data::new(AppData { cache: cache.clone() }))
                .service(index)
        })
        .bind(("127.0.0.1", 8080))?;
        server.run().await?;
        Ok::<_, Error>(())
    })?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct Query {
    #[serde(rename = "resourceVersion")]
    resource_version: Option<ResourceVersion>,
}

#[actix_web::get("/watch")]
async fn index(query: web::Query<Query>, appdata: web::Data<AppData>) -> impl Responder {
    let cache = appdata.get_ref().cache.read().await;
    let stream = cache.stream(query.resource_version);
    let stream = stream.map(|evt| {
        let evt = evt.unwrap();
        let mut vec = serde_json::to_vec(&evt).unwrap();
        vec.push(b'\n');
        Ok::<_, std::io::Error>(Bytes::from(vec))
    });
    let ret = BodyStream::new(stream);
    HttpResponse::Ok().body(ret)
}
