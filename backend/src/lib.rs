use std::sync::{Arc, Mutex};

use crate::io::file_in_memory::FileInMemory;
use crate::model::model::{DataType, DataValue, Model, Vector, Record, Field};
use crate::model::datatypes::get_data_types;
use serde_json::Value;
use std::any::Any;


pub mod model;
pub mod io;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


pub fn get_name() -> String {
    return String::from("alex");
}

pub struct DataRecord {
    pub id: Vector,
    pub column: String,
    pub fields: Vec<DataFieldValue>,
}

pub struct DataFieldValue {
    pub value: Box<dyn DataValue>,
    pub reference: Option<Vector>,
}

pub struct DB {
    file_in_memory: FileInMemory,

    data_types: Vec<Arc<dyn DataType + Sync + Send>>,           //todo: why Arc?
}

impl DB {
    pub fn open(path: &str) -> DB {
        let mut db = DB {
            file_in_memory: FileInMemory::new(path.to_string()),
            data_types: get_data_types(),
        };
        db.sync();
        db
    }

    pub fn sync(&mut self) {
        self.file_in_memory.sync()
    }

    pub fn put_value(&mut self, id: Vector, field_idx: u32, value: &Value) {
        let data_provider = &mut self.file_in_memory;
        let mut data_value: Option<Box<dyn Any + 'static>> = None;
        let mut data_field: Option<Field> = None;
        if let Some(m) = data_provider.get_model().lock().unwrap().as_ref() {
            let rec = m.records.iter().find(|r| r.position == id).unwrap();
            let field = &rec.fields[field_idx as usize];
            for dt in &self.data_types {
                match dt.value_from_json(field, value) {
                    Some(value) => {
                        data_value = Some(value);
                        data_field = Some(field.clone());
                    }
                    None => ()
                }
            }
        }
        if let Some(value) = data_value {
            for dt in &self.data_types {
                dt.write_back(&data_field.as_ref().unwrap(), &value, data_provider);
            }
            data_provider.mark_dirty();

        }
    }

    pub fn get_records_count(&self) -> usize {
        self.get_model().lock().unwrap().as_ref().map_or(0, |m| m.records.len())
    }

    pub fn get_records(&self) -> Vec<DataRecord> {
        self.get_model().lock().unwrap().as_ref().map(|m| {
            m.records.iter()
                .map(|r| self.to_data_record(r))
                .collect()
        }).unwrap_or(vec![])
    }

    pub fn get_record(&self, idx: usize) -> Option<DataRecord> {
        self.get_model().lock().unwrap().as_ref().map(|m| {
            let rec = &m.records[idx];
            self.to_data_record(&rec)
        })
    }

    fn to_data_record(&self, rec: &Record) -> DataRecord {
        let mut data_values: Vec<DataFieldValue> = vec![];

        for field in &rec.fields {
            for data_type in &self.data_types {
                match data_type.parse(&field, &self.file_in_memory) {
                    Some(val) => {
                        data_values.push(DataFieldValue {
                            value: val,
                            reference: field.ref_to_record,
                        });
                        break;
                    }
                    _ => continue
                }
            }
        }

        DataRecord {
            id: rec.position,
            column: rec.column.clone(),
            fields: data_values,
        }
    }

    fn get_model(&self) -> &Mutex<Option<Model>> {
        self.file_in_memory.get_model()
    }
}



/*

Record {
    DataValue[]
}

struct DB impls DataProvider {
    model (mutable)
    file

    image in memory

    fn sync() {       // time to time (?) save and load
    }

    fun get_records_count() -> u32

    fun get_record(idx: u32) -> Option<Record>

    fun find_record(row: u8, rows: u8, col: u8, cols: u8) -> Option<Record>

    fun find_record(value: DataValue) -> Option<Record>

}


 */


/*

    int - count             0xce1100            0xce1
    float - fillness        0xf100d0            0xf1d
    float - rgb             0xf10a70            0xf17
    int - rgb               0x127000            0x127

    boolean                 0xb001ea            0xb00
    image - default         0x000000            0x000

    string - 3x5 font       0xabc305            0xabc



    ***
     *   - int - count
    ***


    ***
    **  - float - flood
    *

    ***
    *    - color
    ***

    ***
    * *   - int - color
    ***

    ***
    **    - float, color
    ***

    (empty) - image

    *
    **     - boolean
    **

     *
    * *    - abc
    * *

     *
    ***    - ref
     *

     *
    * *        - direction
     *


    read_char(data_image, x, y) -> Option<char>

    15 bits, not bytes! == 2 byte key

 */
