use crate::model::model::{DataValue, DataType, Field, DataProvider, FieldType};
use crate::model::colors::Color;
use std::collections::HashMap;
use serde_json::Value;
use std::any::Any;

pub const PIE_TYPE: FieldType = FieldType(0b_111_101_110);


pub(crate) struct PieDataValue {
    value: HashMap<String, f32>,
}

impl DataValue for PieDataValue {
    fn get_type_name(&self) -> String {
        String::from("pie")
    }

    fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{");
        let mut add_comma = false;
        for (key, value) in &self.value {
            if add_comma {
                out.push_str(", ");
            }
            out.push_str(&*format!("\"{}\": {:}", key, value));
            add_comma = true;
        }

        out.push_str("}");
        out
    }
}

pub(crate) struct PieDataType;

impl DataType for PieDataType {
    fn parse(&self, field: &Field, data_provider: &dyn DataProvider) -> Option<Box<dyn DataValue>> {
        if field.field_type != PIE_TYPE { return None; }

        let image = data_provider.load(&field.data_start, &field.data_end);
        let mut counters_per_color: HashMap<String, i32> = HashMap::new();
        let mut pixels = 0;
        for x in 0..image.width() {
            for y in 0..image.height() {
                pixels += 1;
                let pix = image.get_pixel(x, y);
                let key = format!("#{:02X?}{:02X?}{:02X?}", pix[0], pix[1], pix[2]);
                match counters_per_color.get_mut(&key) {
                    None => { counters_per_color.insert(key, 1); }
                    Some(value) => (*value) += 1
                };
            }
        }
        return Some(Box::new(PieDataValue {
            value: counters_per_color.drain().map(|(key, value)| (key, (value as f32) / (pixels as f32))).collect()
        }));
    }

    fn value_from_json(&self, field: &Field, json: &Value) -> Option<Box<dyn Any + 'static>> {
        if field.field_type != PIE_TYPE { return None; }
        if let Value::Object(map) = json {
            let map = map.iter().map(|(key, val)| { (key.clone(), (val.as_f64()).unwrap() as f32) }).collect();
            Some(Box::new(PieDataValue {
                value: map
            }))
        } else {
            None
        }
    }

    fn write_back(&self, field: &Field, value: &Box<dyn Any>, data_provider: &mut dyn DataProvider) {
        if field.field_type != PIE_TYPE { return }
        let mut image = data_provider.load_mut(&field.data_start, &field.data_end);
        let pie_value: &PieDataValue = value.as_ref().downcast_ref::<PieDataValue>().unwrap();
        image.clear();
        let total_pixels = image.width() * image.height();
        let mut xx = 0;
        let mut yy = 0;
        for (k,value) in &pie_value.value {
            let mut pixels = (value * total_pixels as f32).floor() as u32;
            if pixels == 0 { continue; }
            let color = image::Rgba([
                u8::from_str_radix(&k[1..3], 16).unwrap(),
                u8::from_str_radix(&k[3..5], 16).unwrap(),
                u8::from_str_radix(&k[5..7], 16).unwrap(),
                255
            ]);
            'out: while xx < image.width() {
                while yy < image.height() {
                    image.set_pixel(xx, yy, &color);
                    yy += 1;
                    pixels -= 1;
                    if pixels == 0 {
                        break 'out;
                    }
                }
                yy = 0;
                xx += 1;
            }
        }
    }
}

