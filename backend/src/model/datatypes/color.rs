use crate::model::model::{Field, FieldType, DataType, DataValue, DataError, IncompatibleError};
use crate::image::ImageView;

pub const COLOR_TYPE: FieldType = FieldType(0b_111_100_111);

pub(crate) struct ColorDataType;

impl DataType for ColorDataType {

    fn read(&self, image: &ImageView, _: &Field) -> Result<DataValue, DataError> {
        Ok(DataValue::Color { value: image.get_pixel(0, 0) })
    }

    fn write(&self, image: &mut ImageView, _: &Field, value: DataValue) -> Result<(), DataError> {
        let color = get_matched!(value, DataValue::Color { value })?;
        image.fill(&color);
        Ok(())
    }
}
