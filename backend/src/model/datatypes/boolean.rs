use crate::model::model::{DataValue, DataType, Field, DataProvider, FieldType};
use crate::model::colors::Color;

pub const BOOL_TYPE: FieldType = FieldType(0b_100_110_110);


pub(crate) struct BooleanDataValue {
    value: bool
}

impl DataValue for BooleanDataValue {
    fn get_type_name(&self) -> String {
        String::from("boolean")
    }

    fn to_json(&self) -> String {
        match self.value {
            true => String::from("true"),
            false => String::from("false")
        }
    }
}

pub(crate) struct BooleanDataType;

impl DataType for BooleanDataType {

    fn parse(&self, field: &Field, data_provider: &dyn DataProvider) -> Option<Box<dyn DataValue>> {
        if field.field_type != BOOL_TYPE { return None }

        let image = data_provider.load(&field.data_start, &field.data_start);
        let rgba = image.get_pixel(0, 0);
        return Some(Box::from(BooleanDataValue {
            value: rgba.is_data()
        }))
    }
}

