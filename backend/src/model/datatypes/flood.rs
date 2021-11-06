use crate::model::model::{Field, FieldType, DataType, DataValue, DataError};
use crate::image::ImageView;

pub const FLOOD_TYPE: FieldType = FieldType(0b_111_110_100);

pub(crate) struct FloodDataType;

impl DataType for FloodDataType {

    fn read(&self, image: &ImageView, _: &Field) -> Result<DataValue, DataError> {
        let mut pixels = 0;
        let mut flood_pixels = 0;
        for x in 0..image.width {
            for y in 0..image.height {
                pixels += 1;
                if image.get_pixel(x, y).is_data() {
                    flood_pixels += 1;
                }
            }
        }
        Ok(DataValue::Float { value: (flood_pixels as f32) / (pixels as f32)})
    }
}

