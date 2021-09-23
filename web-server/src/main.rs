use warp::{Filter};
use badbee_backend::{DB, DataRecord, DataFieldValue};
use badbee_backend::model::model::Vector;
use std::collections::HashMap;
use warp::reply::{json, Json, Reply, Response};
use maplit::hashmap;
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};
use std::thread::sleep;
use std::time::Duration;

/*

GET /:db/records.[json|html][?q]

q =

    x=1&y=2

    col=1&cols=5  (col 1/5)

    type=x&value=y

    ?include_references

PATCH /:db/records/x/y/0 {type: "", value: ""}


 */

fn get_all_records(db: &DB) -> Vec<DataRecord> { db.get_records() }

fn vec2id(vector: Vector) -> String {
    format!("{}/{}", vector.x, vector.y)
}

fn get_records_json(records: Vec<DataRecord>) -> warp::http::Response<String> {
    let mut out = String::new();
    out.push_str("[");
    let mut ocomma = false;
    for rec in records.iter() {
        if ocomma { out.push_str(",")}
        out.push_str("{");
        out.push_str(&format!("\"id\": \"{}\", \"column\": \"{}\", \"fields\": [", vec2id(rec.id), rec.column));
        let mut comma = false;
        for field in rec.fields.iter() {
            if comma { out.push_str(",")}
            out.push_str("{");
            out.push_str(&format!("\"type\": \"{}\", \"value\": {}", field.value.get_type_name(), field.value.to_json()));
            if let Some(v) = field.reference {
                out.push_str(&format!(", \"reference\": \"{}\"", vec2id(v)))
            }
            out.push_str("}");
            comma = true
        }
        out.push_str("]}");
        ocomma = true
    }
    out.push_str("]");
    return warp::http::Response::builder()
        .header("Content-Type", "application/json")
        .body(out)
        .unwrap()

}

#[tokio::main]
async fn main() {
    let mut dbs = HashMap::new();
    dbs.insert("adtt", Arc::new(Mutex::new(DB::open("db2.png"))));      //todo: why Arc, Mutex?
    dbs.insert("example", Arc::new(Mutex::new(DB::open("db1.png"))));

    let sync_dbs = dbs.clone();

    let do_sync_all = move || {
        loop {
            for (_, mut v) in &sync_dbs {
                v.lock().unwrap().sync();
            }
            sleep(Duration::from_secs(5))
        }
    };
    std::thread::spawn(do_sync_all);



    fn with_dbs(dbs: HashMap<&str, Arc<Mutex<DB>>>) -> impl Filter<Extract=(HashMap<&str, Arc<Mutex<DB>>>, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || dbs.clone())
    }

    let with_dbs_filter = with_dbs(dbs);

    let get_records = warp::path!(String / "records.json")
        .and(with_dbs_filter.clone())
        .map(|dbname: String, dbs: HashMap<&str, Arc<Mutex<DB>>>| {
            let db = &dbs[dbname.as_str()];
            let response = get_records_json(get_all_records(&db.lock().unwrap()));
            Ok(response)
        });

    let put_field = warp::put()
        .and(warp::path!(String / "records" / u32 / u32 / u32))
        .and(with_dbs_filter.clone())
        .and(warp::body::json())
        .map(|dbname: String, x: u32, y: u32, fi: u32, dbs: HashMap<&str, Arc<Mutex<DB>>>, json: Value| {
            if let Value::Object(obj) = json {
                let db = &dbs[dbname.as_str()];
                println!("{} {} {}", x, y, fi);
                println!("{}", obj["value"]);
                let db = &mut db.lock().unwrap();

                db.put_value(Vector { x, y }, fi, &obj["value"])

            }
            Ok("Ok")
        })
    ;


    let routes = get_records.or(put_field);
    let static_files = warp::get().and(warp::path::end()).and(warp::fs::dir("static"));
    warp::serve(routes.or(static_files))
        .run(([127, 0, 0, 1], 3030))
        .await;

}