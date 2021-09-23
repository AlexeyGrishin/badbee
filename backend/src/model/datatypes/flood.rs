use crate::model::model::{DataValue, DataType, Field, DataProvider, FieldType};
use crate::model::colors::Color;

pub const FLOOD_TYPE: FieldType = FieldType(0b_111_110_100);


pub(crate) struct FloodDataValue {
    value: f64
}

impl DataValue for FloodDataValue {
    fn get_type_name(&self) -> String {
        String::from("float")
    }

    fn to_json(&self) -> String {
        format!("{:}", self.value)
    }
}

pub(crate) struct FloodDataType;

impl DataType for FloodDataType {

    fn parse(&self, field: &Field, data_provider: &dyn DataProvider) -> Option<Box<dyn DataValue>> {
        if field.field_type != FLOOD_TYPE { return None }

        let image = data_provider.load(&field.data_start, &field.data_end);
        let mut pixels = 0;
        let mut flood_pixels = 0;
        for x in 0..image.width() {
            for y in 0..image.height() {
                pixels += 1;
                if image.get_pixel(x, y).is_data() {
                    flood_pixels += 1
                }
            }
        }
        return Some(Box::from(FloodDataValue {
            value: (flood_pixels as f64)/(pixels as f64)
        }))
    }
}

