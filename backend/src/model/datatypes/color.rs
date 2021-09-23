use crate::model::model::{DataValue, DataType, Field, DataProvider, FieldType};
use crate::model::colors::Color;
use image::Rgba;

pub const COLOR_TYPE: FieldType = FieldType(0b_111_100_111);


pub(crate) struct ColorDataValue {
    value: Rgba<u8>
}

impl DataValue for ColorDataValue {
    fn get_type_name(&self) -> String {
        String::from("color")
    }

    fn to_json(&self) -> String {
        format!("\"#{:02X?}{:02X?}{:02X?}\"", self.value[0], self.value[1], self.value[2])
    }
}

pub(crate) struct ColorDataType;

impl DataType for ColorDataType {

    fn parse(&self, field: &Field, data_provider: &dyn DataProvider) -> Option<Box<dyn DataValue>> {
        if field.field_type != COLOR_TYPE { return None }

        let image = data_provider.load(&field.data_start, &field.data_start);
        let rgba = image.get_pixel(0, 0);
        return Some(Box::from(ColorDataValue {
            value: rgba.clone()
        }))
    }
}

