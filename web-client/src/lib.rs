mod dog_view;
mod api;
mod dog_model;

use sauron::html::text;
use sauron::prelude::*;
use sauron::js_sys::TypeError;
use sauron::{node, Cmd, Application, Node, Program};
use crate::api::RecordsQuery;
use crate::dog_view::dog_view;
use crate::dog_model::DogModel;

#[derive(Debug)]
pub enum Msg {
    DBListLoaded(Result<api::DBList, TypeError>),
    DBInfoLoaded(Result<api::Model, TypeError>),
    DBSelected(String),
    RecordsLoaded(Result<api::RecordsList, TypeError>),

    DogRenamed { id: String, new_name: String },

    DBRecordPatched(Result<(), TypeError>),

    PrevPage, NextPage,

    ToggleParents { id: String }
}

pub struct App {
    db_name: Option<String>,
    offset: u32,
    limit: u32,
    total: u32,
    loading_time: u32,

    records: Vec<DogModel>,
    db_names: Vec<String>,

    load_refs_requested_for: Option<String>
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
            db_names: vec![],
            load_refs_requested_for: None
        }
    }

    fn load_records(&mut self) -> Cmd<Self, Msg> {
        self.fetch_records(&self.db_name.as_ref().unwrap(), RecordsQuery::column_subset("%23008000".to_string(), self.offset + 1, self.limit))
    }
}

impl Application<Msg> for App {
    fn init(&mut self) -> Cmd<Self, Msg> {
        console_log::init_with_level(log::Level::Trace).unwrap();
        self.fetch_db_list()
    }

    fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
        match msg {
            Msg::DBListLoaded(list) => {
                //log::info!("{:?}", list);
                self.db_names = list.unwrap();
                return Cmd::from(Effects::with_local(vec![Msg::DBSelected(self.db_names.last().unwrap().clone())]));
            }
            Msg::DBInfoLoaded(info) => {
                let model = info.unwrap();
                self.total = model.records - 2;
                self.offset = 0;
                self.limit = 100;
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
                let dogs = list.unwrap().iter().map(|r| DogModel::new_from_record(r)).collect();
                if let Some(id) = &self.load_refs_requested_for {
                    if let Some(dog) = self.records.iter_mut().find(|it| &it.id == id ) {
                        dog.loaded_parents = dogs;
                    }
                    self.load_refs_requested_for = None;
                } else {
                    self.records = dogs;
                }
            }
            Msg::DogRenamed { id, new_name } => {
                return self.patch_record(
                    self.db_name.as_ref().unwrap(),
                    id,
                    DogModel::name_field_idx(),
                    DogModel::name_field_value(&new_name)
                );
            },
            Msg::DBRecordPatched(_result) => {
                //log::info!("{:?}", result);
            },
            Msg::NextPage => {
                self.offset += 100;
                return self.load_records();
            },
            Msg::PrevPage => {
                self.offset -= 100;
                return self.load_records();
            },
            Msg::ToggleParents { id } => {
                if let Some(dog) = self.records.iter_mut().find(|it| it.id == id ) {
                    dog.shown_parents = !dog.shown_parents;
                    if dog.loaded_parents.is_empty() {
                        self.load_refs_requested_for = Some(id);
                        let mut ids: Vec<String> = vec![];
                        if let Some(papa_id) = dog.papa_id.as_ref() {
                            ids.push(papa_id.clone());
                        }
                        if let Some(mama_id) = dog.mama_id.as_ref() {
                            ids.push(mama_id.clone());
                        }
                        return self.fetch_records(&self.db_name.as_ref().unwrap(), RecordsQuery::by_ids(ids));
                    }
                }

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
                        <button class="next" on_click=|_| { Msg::NextPage } disabled={self.offset + 100 >= self.total}>{text("Next page >")}</button>
                    </div>
                </div>
                <div class="records">
                    { for dog in &self.records {
                        dog_view(dog.clone())
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