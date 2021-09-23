use badbee_backend::{get_name, DB};
use std::thread::sleep;
use std::time::Duration;
use badbee_backend::io::bitmap_font::BitmapFont;

fn main() {
    println!("Hello, {}!", get_name());

    let font = BitmapFont::open3x5("font1.png", "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_+-*");

    let mut db = DB::open("db1.png");

    loop {
        db.sync();
        let count = db.get_records_count();
        println!("Count = {}", count);
        for i in 0..count {
            let rec = db.get_record(i).unwrap();
            println!("REC #{:?}", rec.id);

            for field in &rec.fields {
                println!(" Type = \"{}\" Value = {}", field.value.get_type_name(), field.value.to_json());
                if field.reference != None {
                    println!("  References to #{:?}", field.reference.unwrap())
                }
            }
        }
        break
        sleep(Duration::from_secs(5))

    }
}
