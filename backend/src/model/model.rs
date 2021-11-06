use crate::image::{ImageView};
use std::collections::HashMap;
use crate::model::colors::RGB;
use std::time::Duration;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct FieldType(pub u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Vector2D {
    pub x: u32,
    pub y: u32,
}

impl Vector2D {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub field_type: FieldType,
    pub data_start: Vector2D,
    pub(crate) data_end: Vector2D,
    pub(crate) type_start: Vector2D,
    pub ref_to_record: Option<Vector2D>,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub position: Vector2D,
    pub fields: Vec<Field>,
    pub column: String,
    pub(crate) rb_position: Vector2D,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub records: Vec<Record>,
    pub loading_time: Duration,

    by_id: HashMap<Vector2D, usize>,
}

impl Model {
    pub fn new() -> Self {
        Self { records: vec![], by_id: HashMap::new(), loading_time: Duration::from_secs(0) }
    }

    pub fn add_record(&mut self, rec: &Record) {
        self.records.push(rec.clone());
        self.by_id.insert(rec.position, self.records.len() - 1);
    }

    pub fn insert_record(&mut self, idx: usize, rec: &Record) {
        self.records.insert(idx, rec.clone());
        //let's recalculate all...
        for idx in 0..self.records.len() {
            self.by_id.insert(self.records[idx].position, idx);
        }
    }

    pub fn get_by_id(&self, x: u32, y: u32) -> Option<&Record> {
        self.by_id.get(&Vector2D::new(x, y)).map(|i| &self.records[*i])
    }
}

#[derive(Debug)]
pub enum IncompatibleError {
    InvalidDataType,
    InvalidSize,
    CannotParseValue(String),
}

#[derive(Debug)]
pub enum DataError {
    Incompatible(IncompatibleError),
    UnknownType(FieldType),
    NotImplemented,
    NotFound,
}

#[derive(Debug)]
pub enum DataValue {
    Null,
    Boolean { value: bool },
    Int { value: i32 },
    Float { value: f32 },
    String { value: String },
    Color { value: RGB },
    Image { width: u32, height: u32, data_url: String },
    Histogram { value: HashMap<RGB, f32> },

    Custom { subtype: String, value: String },
}

macro_rules! get_matched {
    ($val: expr, $subtype: pat) => {
        match $val {
            $subtype => Ok($val),
            _ => Err(DataError::Incompatible(IncompatibleError::InvalidDataType))
        }
    }
}

pub trait DataType {
    fn read(&self, image: &ImageView, field: &Field) -> Result<DataValue, DataError>;

    fn write(&self, _image: &mut ImageView, _field: &Field, _value: DataValue) -> Result<(), DataError> {
        Err(DataError::NotImplemented)
    }
}

impl From<DataError> for String {
    fn from(de: DataError) -> Self {
        match de {
            DataError::Incompatible(er) => format!("Incompatible: {:?}", er),
            DataError::UnknownType(dt) => format!("Unknown data type {:?}", dt),
            DataError::NotImplemented => String::from("Not implemented!"),
            DataError::NotFound => String::from("Not found"),
        }
    }
}

impl std::ops::AddAssign<Vector2D> for Vector2D {
    fn add_assign(&mut self, rhs: Vector2D) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}