use crate::model::model::DataType;
use std::sync::Arc;

pub(crate) mod boolean;
pub(crate) mod flood;
pub(crate) mod image;
pub(crate) mod abc;
pub(crate) mod color;
pub(crate) mod counter;
pub(crate) mod reference;
pub(crate) mod pie;

pub fn get_data_types() -> Vec<Arc<dyn DataType + Sync + Send>> {
    vec![
        Arc::new(boolean::BooleanDataType {}),      //todo: why Arc? they do not have any state
        Arc::new(flood::FloodDataType {}),
        Arc::new(abc::ABCDataType::new()),
        Arc::new(color::ColorDataType {}),
        Arc::new(counter::CounterDataType {}),
        Arc::new(pie::PieDataType {}),
        Arc::new(image::ImageDataType {}),
    ]
}