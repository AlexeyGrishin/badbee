mod api;
mod record_view;
mod field_view;

use sauron::html::text;
use sauron::prelude::*;
use sauron::js_sys::TypeError;
use sauron::{node, Cmd, Application, Node, Program};
use crate::api::{Record, RecordsQuery};
use record_view::record_view;
use serde_json::json;


#[derive(Debug)]
pub enum Msg {
    DBListLoaded(Result<api::DBList, TypeError>),
    DBInfoLoaded(Result<api::Model, TypeError>),
    DBSelected(String),
    RecordsLoaded(Result<api::RecordsList, TypeError>),
    DBRecordPatched(Result<(), TypeError>),

    PrevPage, NextPage,

    PatchRequested { id: String, fi: usize, new_value: serde_json::Value },

    Noop
}

pub struct App {
    db_name: Option<String>,
    offset: u32,
    limit: u32,
    total: u32,
    loading_time: u32,

    records: Vec<Record>,
    db_names: Vec<String>
}


impl App {
    pub fn new() -> Self {
        App {
            db_name: None,
            offset: 0,
            limit: 0,
            total: 0,
            loading_time: 0,
            records: vec![],
            db_names: vec![]
        }
    }

    fn load_records(&mut self) -> Cmd<Self, Msg> {
        self.fetch_records(&self.db_name.as_ref().unwrap(), RecordsQuery::subset( self.offset, self.limit))
    }
}

impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        console_log::init_with_level(log::Level::Trace).unwrap();
        self.fetch_db_list()
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::Noop => {},
            Msg::DBListLoaded(list) => {
                //log::info!("{:?}", list);
                self.db_names = list.unwrap();
                return Cmd::from(Effects::with_local(vec![Msg::DBSelected(self.db_names.last().unwrap().clone())]));
            }
            Msg::DBInfoLoaded(info) => {
                let model = info.unwrap();
                self.total = model.records;
                self.offset = 0;
                self.limit = 30;
                log::info!("Model {} loaded in {} ms", self.db_name.as_ref().unwrap(), model.loading_time);
                self.loading_time = model.loading_time;
                return self.load_records();
            }
            Msg::DBSelected(db_name) => {
                self.db_name = Some(db_name.clone());
                return self.fetch_db_info(db_name);
            }
            Msg::RecordsLoaded(list) => {
                //log::info!("{:?}", list);
                self.records = list.unwrap()
            }
            Msg::DBRecordPatched(_result) => {
                //log::info!("{:?}", result);
            },
            Msg::NextPage => {
                self.offset += 30;
                return self.load_records();
            },
            Msg::PrevPage => {
                self.offset -= 30;
                return self.load_records();
            },
            Msg::PatchRequested { id, fi, new_value } => {
                return self.patch_record(self.db_name.as_ref().unwrap(), id, fi as u32, json!({"value": new_value}))
            }
        }
        Cmd::none().should_update_view(true)
    }

    fn view(&self) -> Node<Msg> {
        node!(
            <main>
                <div class="db-selector">
                    <select on_change=|e| { Msg::DBSelected(e.value) }>
                        <option disabled={true} selected value>{text(" -- select an option --")} </option>
                        { for db in &self.db_names {
                            let option_selected = if Some(db) == self.db_name.as_ref() {
                                selected("")
                            } else {
                                empty_attr()
                            };
                            node! { <option value={db} { option_selected }>{text(db)}</option> }

                        } }
                    </select>
                    { view_if(self.db_name.is_some(), node! {
                        <span class="loading-time"> { text(format!("DB loaded in {} ms", self.loading_time)) } </span>
                    } ) }
                    <div class="paging">
                        <button class="prev" on_click=|_| { Msg::PrevPage } disabled={self.offset <= 0} > {text("< Prev page")} </button>
                        <button class="next" on_click=|_| { Msg::NextPage } disabled={self.offset + 30 >= self.total}>{text("Next page >")}</button>
                    </div>
                </div>
                <div class="records">
                    { for record in &self.records {
                        record_view(record.clone())
                    }}
                </div>
            </main>
        )
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    Program::mount_to_body(App::new());
}