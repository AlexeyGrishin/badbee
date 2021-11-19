mod json;
mod handlers;

use warp::{Filter};
use std::collections::HashMap;
use serde_derive::Deserialize;
use std::time::Duration;
use badbee_backend::db::DBHandle;
use crate::handlers::{get_records_handler, put_field_handler, get_model_handler, clone_record_handler, get_dbs_handler};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::signal;

pub type DBMAP = Arc<Mutex<HashMap<String, DBHandle>>>;

#[derive(Deserialize)]
pub struct RecordsQuery {
    offset: Option<u32>,
    limit: Option<u32>,
    column: Option<String>,
    ids: Option<String>,
    //comma-separated
    embed_refs: Option<bool>,
}

#[tokio::main]
async fn main() {
    stderrlog::new().verbosity(2).init().unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let dbs: DBMAP = Arc::new(Mutex::new(HashMap::new()));

    match std::env::var("DB_FILE") {
        Ok(value) => {
            log::info!("Load db specified in DB_NAME env var ({})", value.clone());
            dbs.lock().await.insert(value.clone(), DBHandle::run_in_background(format!("db/{}", value).as_str()));
        }
        Err(_) => {
            log::info!("Load default db");
            dbs.lock().await.insert("db".to_string(), DBHandle::run_in_background("db/db.png"));
        }
    }

    let sync_dbs = dbs.clone();
    let shutdown_dbs = dbs.clone();

    let periodical_sync = tokio::spawn(async move {
        loop {
            for (_, v) in sync_dbs.lock().await.iter() {
                v.sync().await;
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });


    fn with_dbs(dbs: DBMAP) -> impl Filter<Extract=(DBMAP, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || dbs.clone())
    }

    let with_dbs_filter = with_dbs(dbs);

    let get_dbs = warp::path!("dbs.json")
        .and(with_dbs_filter.clone())
        .and_then(get_dbs_handler);

    let get_records = warp::path!(String / "records.json")
        .and(warp::query())
        .and(with_dbs_filter.clone())
        .and_then(get_records_handler);

    let get_model = warp::path!(String / "model.json")
        .and(with_dbs_filter.clone())
        .and_then(get_model_handler);

    let put_field = warp::put()
        .and(warp::path!(String / "records" / u32 / u32 / u32))
        .and(with_dbs_filter.clone())
        .and(warp::body::json())
        .and_then(put_field_handler)
        ;

    let clone_record = warp::post()
        .and(warp::path!(String / "records" / u32 / u32 / "clone"))
        .and(with_dbs_filter.clone())
        .and_then(clone_record_handler);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Content-Type"])
        .allow_methods(vec!["POST", "GET", "PUT", "PATCH", "DELETE"])
        .build();

    let routes = get_dbs
        .or(get_records)
        .or(put_field)
        .or(get_model)
        .or(clone_record);
    let static_files = warp::get().and(warp::fs::dir("static"));

    let (_, server) = warp::serve(routes.or(static_files).with(cors))
        .bind_with_graceful_shutdown(([0, 0, 0, 0], 3030), async { shutdown_rx.await.ok(); });

    tokio::task::spawn(server);
    log::info!("Listen to 0.0.0.0:3030. Waiting for ctrl-c");
    signal::ctrl_c().await.expect("failed to listen for event");
    log::info!("received ctrl-c event");

    shutdown_tx.send(()).unwrap();
    log::info!("stopped web");
    periodical_sync.abort();
    log::info!("stopped periodical");
    for db_handle in shutdown_dbs.lock().await.values() {
        db_handle.sync().await;
        db_handle.shutdown();
    }
    log::info!("stopped dbs");


}