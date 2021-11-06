use image::{Rgba, Rgb, Pixel};

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.r == 255 && self.g == 255 && self.b == 255
    }

    pub(crate) fn is_meta(&self) -> bool {
        self.r == 0xBA && self.g == 0xDB && self.b == 0xEE
    }

    pub(crate) fn is_data(&self) -> bool {
        !self.is_blank() && !self.is_meta()
    }

    pub fn to_hex_color(&self) -> String {
        format!("#{:02X?}{:02X?}{:02X?}", self.r, self.g, self.b)
    }
}

pub const BLANK: RGB = RGB { r: 255, g: 255, b: 255 };

impl From<&String> for RGB {
    fn from(s: &String) -> Self {
        Self::new(
            u8::from_str_radix(&s[1..3], 16).unwrap(),
            u8::from_str_radix(&s[3..5], 16).unwrap(),
            u8::from_str_radix(&s[5..7], 16).unwrap()
        )
    }
}

impl From<Rgba<u8>> for RGB {
    fn from(rgba: Rgba<u8>) -> Self {
        RGB::new(rgba[0], rgba[1], rgba[2])
    }
}

impl From<&RGB> for Rgba<u8> {
    fn from(rgb: &RGB) -> Self {
        Rgba::from_channels(rgb.r, rgb.g, rgb.b, 255)
    }
}


impl From<RGB> for Rgb<u8> {
    fn from(rgb: RGB) -> Self {
        Self::from_channels(rgb.r, rgb.g, rgb.b, 255)
    }
}