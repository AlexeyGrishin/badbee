use crate::model::model::{Field, FieldType, DataType, DataValue, DataError, IncompatibleError};
use std::collections::HashMap;
use crate::image::{ImageView};
use crate::model::colors::RGB;

pub const PIE_TYPE: FieldType = FieldType(0b_111_101_110);

pub(crate) struct PieDataType;

impl DataType for PieDataType {

    fn read(&self, image: &ImageView, _: &Field) -> Result<DataValue, DataError> {
        let mut counters_per_color: HashMap<RGB, i32> = HashMap::new();
        let mut pixels = 0;
        for x in 0..image.width {
            for y in 0..image.height {
                pixels += 1;
                let key = image.get_pixel(x, y);
                match counters_per_color.get_mut(&key) {
                    None => { counters_per_color.insert(key, 1); }
                    Some(value) => (*value) += 1
                };
            }
        }
        Ok(DataValue::Histogram {
            value: counters_per_color.drain().map(|(key, value)| (key, (value as f32) / (pixels as f32))).collect()
        })
    }

    fn write(&self, image: &mut ImageView, _: &Field, value: DataValue) -> Result<(), DataError> {
        let pie_value = get_matched!(value, DataValue::Histogram { value })?;
        image.clear();
        let total_pixels = image.width * image.height;
        let mut xx = 0;
        let mut yy = 0;
        for (color,value) in pie_value {
            let mut pixels = (value * total_pixels as f32).floor() as u32;
            if pixels == 0 { continue; }
            'out: while xx < image.width {
                while yy < image.height {
                    image.set_pixel(xx, yy, color).unwrap();
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
        Ok(())
    }
}