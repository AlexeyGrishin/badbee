use crate::model::model::{Field, FieldType, DataType, DataValue, DataError};
use std::collections::HashSet;
use crate::image::ImageView;

pub const COUNTER_TYPE: FieldType = FieldType(0b_111_010_111);

pub(crate) struct CounterDataType;

impl DataType for CounterDataType {

    fn read(&self, image: &ImageView, _: &Field) -> Result<DataValue, DataError> {
        let mut used_pixels: HashSet<(u32, u32)> = HashSet::new();
        let mut domains_found = 0;

        for y in 0..image.height {
            for x in 0..image.width {
                if image.get_pixel(x, y).is_data() && !used_pixels.contains(&(x, y)) {
                    let mut pixels_to_check: Vec<(u32, u32)> = vec![(x,y)];

                    while !pixels_to_check.is_empty() {
                        let (x,y) = pixels_to_check.pop().unwrap();
                        if used_pixels.contains(&(x, y)) { continue }
                        used_pixels.insert((x,y));
                        if !image.get_pixel(x, y).is_data() { continue }
                        pixels_to_check.push((x+1, y));
                        pixels_to_check.push((x-1, y));
                        pixels_to_check.push((x, y+1));
                        pixels_to_check.push((x, y-1));
                    }
                    domains_found += 1;
                }
            }
        }
        Ok(DataValue::Int {value: domains_found})
    }

}

