use image::Rgba;
use std::ops::Index;
use serde_json::Value;
use std::any::Any;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct FieldType(pub u16);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub(crate) field_type: FieldType,
    pub(crate) data_start: Vector,
    pub(crate) data_end: Vector,
    pub(crate) type_start: Vector,
    pub(crate) ref_to_record: Option<Vector>
}

#[derive(Debug, Clone)]
pub struct Record {
    pub(crate) position: Vector,
    pub(crate) fields: Vec<Field>,
    pub(crate) column: String
}

#[derive(Debug)]
pub struct Model {
    pub(crate) records: Vec<Record>,
}

impl Model {
    pub fn new() -> Self {
        Self { records: vec![] }
    }
}

pub trait DataImage {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> &Rgba<u8>;

    fn get_base64(&self) -> String;
}

pub trait IndexedDataImage: DataImage + Index<(u32, u32), Output = Rgba<u8>> {}

pub trait MutDataImage {
    fn set_pixel(&mut self, x: u32, y: u32, color: &Rgba<u8>);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn clear(&mut self);
}

pub trait DataProvider {
    fn load(&self, from: &Vector, to: &Vector) -> Box<dyn DataImage + '_>;
    fn load_mut(&mut self, from: &Vector, to: &Vector) -> Box<dyn MutDataImage + '_>;
}

pub trait DataValue {
    fn get_type_name(&self) -> String;
    fn to_json(&self) -> String;
}

//todo: reiterate. mb make map, field_type -> DataType. maybe not

pub trait DataType {
    fn parse(&self, field: &Field, data_provider: &dyn DataProvider) -> Option<Box<dyn DataValue>>;

    fn value_from_json(&self, field: &Field, json: &Value) -> Option<Box<dyn Any + 'static>> { None }

    fn write_back(&self, field: &Field, value: &Box<dyn Any>, data_provider: &mut dyn DataProvider) {}
}
