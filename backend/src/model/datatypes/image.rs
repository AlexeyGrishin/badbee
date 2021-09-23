use crate::model::model::{FieldType, DataType, DataValue, DataProvider, Field};

pub const IMAGE_TYPE: FieldType = FieldType(0b_000_000_000);


pub(crate) struct ImageDataValue {
    data_url: String,
    width: u32,
    height: u32
}

impl DataValue for ImageDataValue {
    fn get_type_name(&self) -> String {
        String::from("image")
    }
    fn to_json(&self) -> String {
        format!("{{ \"width\": {}, \"height\": {}, \"data_url\": \"{}\" }}", self.width, self.height, self.data_url)
    }
}

pub(crate) struct ImageDataType;

impl DataType for ImageDataType {

    fn parse(&self, field: &Field, data_provider: &dyn DataProvider) -> Option<Box<dyn DataValue>> {
        //if field.field_type != IMAGE_TYPE { return None }

        let image = data_provider.load(&field.data_start, &field.data_end);
        return Some(Box::from(ImageDataValue {
            width: image.width(), height: image.height(), data_url: format!("data:image/png;base64,{}", image.get_base64())
        }))
    }
}