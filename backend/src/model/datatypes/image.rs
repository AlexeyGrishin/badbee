use crate::model::model::{FieldType, Field, DataType, DataValue, DataError, IncompatibleError};
use crate::image::ImageView;
use image::{ImageFormat, ImageError};
use imageproc::drawing::Canvas;
use base64::DecodeError;

pub const IMAGE_TYPE: FieldType = FieldType(0b_000_000_000);

pub(crate) struct ImageDataType;

impl DataType for ImageDataType {

    fn read(&self, image: &ImageView, _: &Field) -> Result<DataValue, DataError> {
        Ok(DataValue::Image {
            width: image.width,
            height: image.height,
            data_url: format!("data:image/png;base64,{}", image.get_base64())
        })
    }

    fn write(&self, image: &mut ImageView, _field: &Field, value: DataValue) -> Result<(), DataError> {
        match value {
            DataValue::Image { width, height, data_url } => {
                if width != image.width || height != image.height {
                    log::error!("target size is {}x{}, but value size is {}x{}", image.width, image.height, width, height);
                    Result::Err(DataError::Incompatible(IncompatibleError::InvalidSize))
                } else {
                    let bytes = base64::decode(data_url.strip_prefix("data:image/png;base64,").unwrap())?;
                    let temp_image = image::load_from_memory_with_format(&bytes, ImageFormat::Png)?;
                    for x in 0..width {
                        for y in 0..height {
                            image.set_pixel(x, y, temp_image.get_pixel(x, y))?;
                        }
                    }
                    Result::Ok(())
                }

            }
            _ => Result::Err(DataError::Incompatible(IncompatibleError::InvalidDataType))
        }
    }
}

impl From<()> for DataError {
    fn from(_: ()) -> Self {
        DataError::Incompatible(IncompatibleError::InvalidSize)
    }
}

impl From<DecodeError> for DataError {
    fn from(e: DecodeError) -> Self {
        DataError::Incompatible(IncompatibleError::CannotParseValue(e.to_string()))
    }
}

impl From<ImageError> for DataError {
    fn from(ie: ImageError) -> Self {
        log::error!("Error {}", ie);
        DataError::Incompatible(IncompatibleError::CannotParseValue(ie.to_string()))
    }
}