use crate::{DBMAP, RecordsQuery};
use warp::reply::{Json, with_status};
use badbee_backend::db::{DBQuery, DataRecord, DBResult};
use crate::json::{to_json, from_json};
use serde_json::{json, Value};
use warp::{Reply, Rejection};
use badbee_backend::model::model::Vector2D;
use warp::http::StatusCode;
use log::error;

pub async fn get_dbs_handler(dbs: DBMAP) -> Result<impl Reply, Rejection> {
    let db_names: Vec<String> = dbs.lock().await.keys().map(|k| k.clone()).collect();
    Ok(warp::reply::json(&db_names))
}

pub async fn get_model_handler(dbname: String, dbs: DBMAP) -> Result<Box<dyn Reply>, Rejection> {
    if !dbs.lock().await.contains_key(dbname.as_str()) {
        return Ok(Box::new(with_status("Unknown db", StatusCode::NOT_FOUND)));
    }
    let db = &dbs.lock().await[dbname.as_str()];
    match db.get_model().await {
        DBResult::Ok(model) => Ok(Box::new(warp::reply::json(&json!({
            "loading_time": model.loading_time.as_millis() as u32,
            "records": model.records.len(),
            "fields_max": model.records.iter().map(|r| r.fields.len()).max(),
            "fields_min": model.records.iter().map(|r| r.fields.len()).min(),
        })))),
        DBResult::StillLoading(progress) => {
            Ok(
                Box::new(with_status(format!("Still loading model ({}%)", (progress*100.0) as u32), StatusCode::PARTIAL_CONTENT))
            )

        },
        DBResult::Err(error) => {
            error!("ERROR {}", error);
            Ok(Box::new(with_status(error, StatusCode::INTERNAL_SERVER_ERROR)))
        }
    }
}

pub async fn clone_record_handler(dbname: String, x: u32, y: u32, dbs: DBMAP) -> Result<Box<dyn Reply>, Rejection> {
    if !dbs.lock().await.contains_key(dbname.as_str()) {
        return Ok(Box::new(with_status("Unknown db", StatusCode::NOT_FOUND)));
    }
    let db = &dbs.lock().await[dbname.as_str()];
    match db.clone_record(x, y).await {
        DBResult::Ok(record) => Ok(Box::new(get_records_json(vec![record], false))),
        DBResult::StillLoading(progress) => {
            Ok(
                Box::new(with_status(format!("Still loading model ({}%)", (progress*100.0) as u32), StatusCode::PARTIAL_CONTENT))
            )

        },
        DBResult::Err(error) => {
            error!("ERROR {}", error);
            Ok(Box::new(with_status(error, StatusCode::INTERNAL_SERVER_ERROR)))
        }
    }
}

pub async fn get_records_handler(dbname: String, q: RecordsQuery, dbs: DBMAP) -> Result<Box<dyn Reply>, Rejection> {
    if !dbs.lock().await.contains_key(dbname.as_str()) {
        return Ok(Box::new(with_status("Unknown db", StatusCode::NOT_FOUND)));
    }
    let db = &dbs.lock().await[dbname.as_str()];
    let mut query = DBQuery::new();
    if let Some(ids) = q.ids {
        let ids: Vec<String> = ids.split(",")
            .map(|it| it.to_string())
            .filter(|it| it.contains("/"))
            .collect();
        if ids.len() != 0 {
            query.ids(ids.iter().map(|xy| {
                let mut split = xy.split("/");
                Vector2D::new(split.next().unwrap().parse().unwrap(), split.next().unwrap().parse().unwrap())
            }).collect());
        }
    }
    if let Some(offset) = q.offset {
        query.offset(offset);
    }
    if let Some(limit) = q.limit {
        query.limit(limit);
    }
    if let Some(column) = q.column {
        query.column(column);
    }
    match db.get_records(query).await {
        DBResult::Ok(records) => Ok(Box::new(get_records_json(records, q.embed_refs.unwrap_or(false)))),
        DBResult::StillLoading(progress) => {
            Ok(
                Box::new(with_status(format!("Still loading model ({}%)", (progress*100.0) as u32), StatusCode::PARTIAL_CONTENT))
            )

        },
        DBResult::Err(error) => {
            error!("ERROR {}", error);
            Ok(Box::new(with_status(error, StatusCode::INTERNAL_SERVER_ERROR)))
        }
    }
}



pub async fn put_field_handler(dbname: String, x: u32, y: u32, fi: u32, dbs: DBMAP, json: Value) -> Result<impl Reply, Rejection> {
    if !dbs.lock().await.contains_key(dbname.as_str()) {
        return Ok(with_status("Unknown db".to_string(), StatusCode::NOT_FOUND));
    }
    if let Value::Object(ref _obj) = json {
        let db = &dbs.lock().await[dbname.as_str()];
        let value = from_json(&json).unwrap(); //todo: err
        match db.set_field(x, y, fi, value).await {
            DBResult::Ok(_) => Ok(with_status("Ok".to_string(), StatusCode::OK)),
            DBResult::StillLoading(progress) => {
                Ok(
                    with_status(format!("Still loading model ({}%)", (progress*100.0) as u32), StatusCode::PARTIAL_CONTENT)
                )

            },
            DBResult::Err(error) => {
                error!("ERROR {}", error);
                Ok(with_status(error, StatusCode::INTERNAL_SERVER_ERROR))
            }
        }
    } else {
        Ok(with_status("Invalid json".to_string(), StatusCode::INTERNAL_SERVER_ERROR))
    }

}

fn get_records_json(records: Vec<DataRecord>, embed_refs: bool) -> Json {
    let mut jsons = vec![];
    for rec in records.iter() {
        let mut field_jsons = vec![];
        for field in &rec.fields {
            field_jsons.push(to_json(&field, embed_refs))
        }
        let rec_json = json![{
            "id": vec2id(rec.id),
            "column": rec.column,
            "fields": field_jsons
        }];
        jsons.push(rec_json);
    }
    warp::reply::json(&jsons)
}


pub(crate) fn vec2id(vector: Vector2D) -> String {
    format!("{}/{}", vector.x, vector.y)
}
