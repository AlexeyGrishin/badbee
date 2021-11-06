use crate::api::Record;
use serde_json::Value;
use serde_json::json;


#[derive(Clone)]
pub enum Gender {
    Male, Female
}

#[derive(Clone)]
pub struct DogModel {
    pub id: String,
    pub avatar_src: String,
    pub name: String,
    pub count_of_trophys: u32,
    pub gender: Gender,
    pub papa_id: Option<String>,
    pub mama_id: Option<String>,
    pub country_src: String,

    pub loaded_parents: Vec<DogModel>,
    pub shown_parents: bool

}


impl DogModel {

    pub fn name_field_idx() -> u32 { 5 }

    pub fn name_field_value(name: &String) -> Value {
        json!({"value": name})
    }

    pub fn new_from_record(record: &Record) -> Self {
        Self {
            id: record.id.clone(),
            avatar_src: record.fields[0].value.get("data_url").unwrap().as_str().unwrap().to_string(),
            name: record.fields[5].value.as_str().unwrap().to_string(),
            count_of_trophys: record.fields[2].value.as_i64().unwrap() as u32,
            gender: if record.fields[1].value.as_str().unwrap() == "#00A2E8" { Gender::Male } else { Gender::Female },
            papa_id: record.fields[3].reference.clone(),
            mama_id: record.fields[4].reference.clone(),
            country_src: record.fields[7].value.get("data_url").unwrap().as_str().unwrap().to_string(),
            loaded_parents: vec![],
            shown_parents: false
        }
    }

}