use sauron::Cmd;
use sauron::prelude::*;
use sauron::prelude::web_sys::RequestInit;
use serde_derive::Deserialize;
use crate::{App, Msg};


pub type DBList = Vec<String>;

#[derive(Deserialize, Debug, Clone)]
pub struct RecordField {
    #[serde(rename = "type")]
    pub(crate) ftype: Option<String>,
    pub(crate) reference: Option<String>,
    pub(crate) value: serde_json::Value
}

#[derive(Deserialize, Debug, Clone)]
pub struct Record {
    pub(crate) id: String,
    pub(crate) fields: Vec<RecordField>,
    pub(crate) column: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Model {
    pub(crate) records: u32,
    pub(crate) loading_time: u32
}

pub type RecordsList = Vec<Record>;

pub struct RecordsQuery {
    offset: Option<u32>, limit: Option<u32>, column: Option<String>, ids: Option<Vec<String>>
}

impl RecordsQuery {
    pub fn all() -> Self { Self { offset: None, limit: None, column: None, ids: None }}

    pub fn column(name: String) -> Self { Self { offset: None, limit: None, column: Some(name), ids: None }}

    pub fn subset(offset: u32, limit: u32) -> Self { Self { offset: Some(offset), limit: Some(limit), column: None, ids: None}}

    pub fn column_subset(name: String, offset: u32, limit: u32) -> Self { Self { offset: Some(offset), limit: Some(limit), column: Some(name), ids: None}}

    pub fn by_ids(ids: Vec<String>) -> Self { Self { offset: None, limit: None, column: None, ids: Some(ids)}}
}

impl App {

    pub fn fetch_db_list(&self) -> Cmd<Self, Msg> {
        Http::fetch_with_text_response_decoder(
            "/dbs.json",
            |s| Msg::DBListLoaded(Result::Ok(serde_json::from_str(&s).unwrap())),
            |err| Msg::DBListLoaded(Result::Err(err))
        )
    }

    pub fn fetch_db_info(&self, name: String) -> Cmd<Self, Msg> {
        Http::fetch_with_text_response_decoder(
            format!("{}/model.json", name).as_str(),
            |s| Msg::DBInfoLoaded(Result::Ok(serde_json::from_str(&s).unwrap())),
            |err| Msg::DBInfoLoaded(Result::Err(err))
        )
    }

    pub fn patch_record(&self, name: &String, rid: String, fid: u32, body: serde_json::Value) -> Cmd<Self, Msg> {
        Http::fetch_with_request_and_response_decoder(
            format!("{}/records/{}/{}", name, rid, fid).as_str(),
            Some(RequestInit::new()
                .method("PUT")
                .headers(&JsValue::from_serde(&serde_json::json!({ "Content-Type": "application/json"})).unwrap())
                .body(Some(&JsValue::from_str(&body.to_string())))
                .clone()
            ),
            |(_, p)| p.dispatch(Msg::DBRecordPatched(Result::Ok(()))),
            |err| Msg::DBRecordPatched(Result::Err(err))
        )
    }

    pub fn fetch_all_records(&self, name: &String) -> Cmd<Self, Msg> {
        self.fetch_records(name, RecordsQuery::all())
    }

    pub fn fetch_records(&self, name: &String, query: RecordsQuery) -> Cmd<Self, Msg> {
        let mut url = format!("{}/records.json?", name);
        if let Some(offset) = query.offset {
            url += format!("offset={}&", offset).as_str();
        }
        if let Some(limit) = query.limit {
            url += format!("limit={}&", limit).as_str();
        }
        if let Some(column) = query.column {
            url += format!("column={}&", column).as_str();
        }
        if let Some(ids) = query.ids {
            url += "ids=";
            for id in ids {
                url += format!("{},", id).as_str();
            }
            url += "&";
        }
        Http::fetch_with_text_response_decoder(
            url.as_str(),
            |s| Msg::RecordsLoaded(Result::Ok(serde_json::from_str(&s).unwrap())),
            |err| Msg::RecordsLoaded(Result::Err(err))
        )
    }
}