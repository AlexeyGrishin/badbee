use crate::model::model::{FieldType, DataType, DataProvider, DataValue, Field};
use crate::io::bitmap_font::BitmapFont;
use serde_json::Value;
use std::any::Any;

pub const ABC_TYPE: FieldType = FieldType(0b_010_101_101);

pub(crate) struct ABCDataValue {
    string: String
}

impl DataValue for ABCDataValue {
    fn get_type_name(&self) -> String {
        String::from("string")
    }
    fn to_json(&self) -> String {
        return format!("\"{}\"", self.string)
    }
}

pub(crate) struct ABCDataType {
    bitmap_font: BitmapFont,
}

impl ABCDataType {
    pub(crate) fn new() -> ABCDataType {
        return ABCDataType {
            bitmap_font: BitmapFont::open3x5("font1.png", "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_+-*")
        }
    }
}

impl DataType for ABCDataType {
    fn parse(&self, field: &Field, data_provider: &dyn DataProvider) -> Option<Box<dyn DataValue>> {
        if field.field_type != ABC_TYPE { return None }

        let image = data_provider.load(&field.data_start, &field.data_end);
        let mut chars: Vec<char> = vec![];

        let mut start_x = 0;
        let mut start_y = 1;

        while start_x < image.width() && start_y < image.height() {
            let mut char = ' ';
            'out: for dx in 0..=2 {
                for dy in -1..=2 as i32 {
                    match self.bitmap_font.get_char(image.as_ref(), start_x + dx, (start_y as i32 + dy) as u32) {
                        Some(c) => {
                            char = c;
                            start_x += dx;
                            start_x += self.bitmap_font.char_dimensions.0;
                            start_y = (start_y as i32 + dy) as u32;
                            //start_y += self.bitmap_font.char_dimensions.1;
                            break 'out;
                        }
                        None => continue
                    }
                }
            }
            if char == ' ' {
                start_x += self.bitmap_font.char_dimensions.0;
                //start_y += self.bitmap_font.char_dimensions.1;
            }
            chars.push(char);
        }

        let string: String = chars.iter().collect();
        let string = String::from(string.trim());
        return Some(Box::from(ABCDataValue { string }))
    }

    fn value_from_json(&self, field: &Field, json: &Value) -> Option<Box<dyn Any + 'static>> {
        if field.field_type != ABC_TYPE { return None }
        return match json {
            Value::String(str) => Some(Box::new(ABCDataValue { string: str.clone() })),
            _ => None
        }
    }

    fn write_back(&self, field: &Field, value: &Box<dyn Any>, data_provider: &mut dyn DataProvider) {
        if field.field_type != ABC_TYPE { return }
        let mut img = data_provider.load_mut(&field.data_start, &field.data_end);
        let abc_value: &ABCDataValue = value.as_ref().downcast_ref::<ABCDataValue>().unwrap();
        img.clear();
        self.bitmap_font.put_string(img.as_mut(), 1, 1, abc_value.string.as_str());
    }
}