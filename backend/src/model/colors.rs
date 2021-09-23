use image::{Rgba, Pixel};

#[derive(PartialEq)]
pub enum PixelType {
    Blank, Meta, Data
}

pub trait Color {
    fn get_type(&self) -> PixelType;

    fn is_data(&self) -> bool { self.get_type() == PixelType::Data }
    fn is_blank(&self) -> bool { self.get_type() == PixelType::Blank }
    fn is_meta(&self) -> bool { self.get_type() == PixelType::Meta }
}

impl Color for Rgba<u8> {
    fn get_type(&self) -> PixelType {
        match self.channels() {
            [0xFF, 0xFF ,0xFF, _] => PixelType::Blank,
            [0xBA, 0xDB, 0xEE, _] => PixelType::Meta,
            _ => PixelType::Data
        }
    }
}