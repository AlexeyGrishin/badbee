use crate::model::model::{Field, FieldType, DataType, DataValue, DataError, IncompatibleError};
use crate::image::{ImageView};
use crate::model::colors::RGB;

pub const BOOL_TYPE: FieldType = FieldType(0b_100_110_110);

pub(crate) struct BooleanDataType;

impl DataType for BooleanDataType {

    fn read(&self, image: &ImageView, _: &Field) -> Result<DataValue, DataError> {
        Ok(DataValue::Boolean { value: !image.get_pixel(0, 0).is_blank() })
    }

    fn write(&self, image: &mut ImageView, _: &Field, value: DataValue) -> Result<(), DataError> {
        let boolean = get_matched!(value, DataValue::Boolean { value })?;
        if boolean {
            image.fill(&RGB::new(0, 255, 0))
        } else {
            image.clear()
        }
        Ok(())
    }
}
