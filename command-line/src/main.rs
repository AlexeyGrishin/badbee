use std::thread::sleep;
use std::time::Duration;
use badbee_backend::io::bitmap_font::BitmapFont;
use badbee_backend::io::image_io::load_image;
use badbee_backend::image::{ImageView, StorableImage};
use badbee_backend::model::async_model_reader::load_model_into;
use std::io::{Seek, SeekFrom, Read};
use std::os::windows::fs::{FileExt, MetadataExt};
use byteorder::{LittleEndian, ReadBytesExt};
use std::ops::Div;
use badbee_backend::model::model::{Model, DataValue};
use badbee_backend::db::{DBHandle, DBQuery};

#[tokio::main]
async fn main() {
    let handle = DBHandle::run_in_background("db2.png");

    let mut query = DBQuery::new();//.limit(1).build();//.limit(2);
    let model = handle.get_model().await.unwrap();
    for rec in &model.records {
        println!("RECORD {:?}", rec.position);
        for f in &rec.fields {
            println!("FIELD {:?} {:?} {:?}", f.field_type, f.data_start, f.ref_to_record)
        }
    }
    println!();
    let records = handle.get_records(query.clone()).await.unwrap();
    //handle.set_field(records[0].id.x, records[0].id.y, 1, DataValue::Boolean { value: true}).await;
    //handle.sync().await;
    //let records = handle.get_records(query.clone()).await;
    for rec in &records {
        println!("RECORD {:?}", rec.id);
        for field in &rec.fields {
            println!("  FIELD {:?} ref={:?}", field.value, field.reference)
        }
    }


}

