use crate::model::model::{FieldType, Field, DataType, DataValue, DataError, IncompatibleError};
use crate::io::bitmap_font::BitmapFont;
use crate::image::ImageView;

pub const ABC_TYPE: FieldType = FieldType(0b_010_101_101);

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

    fn read(&self, image: &ImageView, _: &Field) -> Result<DataValue, DataError> {
        let mut chars: Vec<char> = vec![];

        let mut start_x = 0;
        let mut start_y = 1;

        while start_x < image.width && start_y < image.height {
            let mut char = ' ';
            'out: for dx in 0..=2 {
                if start_x + dx >= image.width { continue }
                for dy in -1..=2 as i32 {
                    if start_y == 0 && dy < 0 { continue }
                    if (start_y as i32 + dy) as u32  >= image.height { continue }
                    match self.bitmap_font.get_char(image, start_x + dx, (start_y as i32 + dy) as u32) {
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
        Ok(DataValue::String { value: string })
    }

    fn write(&self, image: &mut ImageView, _: &Field, value: DataValue) -> Result<(), DataError> {
        let string = get_matched!(value, DataValue::String { value })?;
        image.clear();
        self.bitmap_font.put_string(image, 1, 1, string.as_str());
        Ok(())
    }
}