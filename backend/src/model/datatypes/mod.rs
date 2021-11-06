use crate::model::model::{DataType, Field, DataError, DataValue, FieldType, IncompatibleError};
use crate::image::ImageView;

pub(crate) mod boolean;
pub(crate) mod flood;
pub(crate) mod image;
pub(crate) mod abc;
pub(crate) mod color;
pub(crate) mod counter;
pub(crate) mod reference;
pub(crate) mod pie;

pub const DEFAULT_TYPE: FieldType = FieldType(0b_000_000_000);

const BOOLEAN_DT: boolean::BooleanDataType = boolean::BooleanDataType {};
const IMAGE_DT: image::ImageDataType = image::ImageDataType {};
const FLOOD_DT: flood::FloodDataType = flood::FloodDataType {};
const COLOR_DT: color::ColorDataType = color::ColorDataType {};
const COUNTER_DT: counter::CounterDataType = counter::CounterDataType {};
const PIE_DT: pie::PieDataType = pie::PieDataType {};

pub struct DataTypes {
    abc_data_type: abc::ABCDataType
}

impl DataTypes {

    pub fn new() -> Self {
        Self {
            abc_data_type: abc::ABCDataType::new()
        }
    }

    pub fn read_casted(&self, image: &ImageView, field: &Field, ftype: FieldType) -> Result<DataValue, DataError> {
        match ftype {
            boolean::BOOL_TYPE => BOOLEAN_DT.read(image, field),
            image::IMAGE_TYPE => IMAGE_DT.read(image, field),
            flood::FLOOD_TYPE => FLOOD_DT.read(image, field),
            abc::ABC_TYPE => self.abc_data_type.read(image, field),
            color::COLOR_TYPE => COLOR_DT.read(image, field),
            counter::COUNTER_TYPE => COUNTER_DT.read(image, field),
            pie::PIE_TYPE => PIE_DT.read(image, field),
            reference::REFERENCE_TYPE => Ok(DataValue::Null),
            _ => Err(DataError::UnknownType(ftype))
        }
    }

    pub fn read(&self, image: &ImageView, field: &Field) -> Result<DataValue, DataError> {
        self.read_casted(image, field, field.field_type)
    }

    pub fn write(&self, image: &mut ImageView, field: &Field, value: DataValue) -> Result<(), DataError> {
        self.write_casted(image, field, field.field_type, value)
    }

    pub fn write_casted(&self, image: &mut ImageView, field: &Field, ftype: FieldType, value: DataValue) -> Result<(), DataError> {
        match ftype {
            boolean::BOOL_TYPE => BOOLEAN_DT.write(image, field, value),
            image::IMAGE_TYPE => IMAGE_DT.write(image, field, value),
            flood::FLOOD_TYPE => FLOOD_DT.write(image, field, value),
            abc::ABC_TYPE => self.abc_data_type.write(image, field, value),
            color::COLOR_TYPE => COLOR_DT.write(image, field, value),
            counter::COUNTER_TYPE => COUNTER_DT.write(image, field, value),
            pie::PIE_TYPE => PIE_DT.write(image, field, value),
            reference::REFERENCE_TYPE => Err(DataError::Incompatible(IncompatibleError::InvalidDataType)),
            _ => Err(DataError::UnknownType(ftype))
        }
    }

    #[allow(dead_code)]
    pub fn get_preferred_type(&self, val: DataValue) -> Option<FieldType> {
        match val {
            DataValue::Boolean { .. } => Some(boolean::BOOL_TYPE),
            DataValue::Int { .. } => Some(counter::COUNTER_TYPE),
            DataValue::Float { .. } => Some(flood::FLOOD_TYPE),
            DataValue::String { .. } => Some(abc::ABC_TYPE),
            DataValue::Color { .. } => Some(color::COLOR_TYPE),
            DataValue::Image { .. } => Some(image::IMAGE_TYPE),
            DataValue::Histogram { .. } => Some(pie::PIE_TYPE),
            DataValue::Custom { .. } => None,
            DataValue::Null => None
        }
    }
}

