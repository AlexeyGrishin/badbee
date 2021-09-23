use image::{DynamicImage, GenericImageView, Rgba};
use crate::model::model::{DataImage, MutDataImage};
use std::collections::HashMap;
use crate::model::colors::Color;


pub struct BitmapFont {
    image: DynamicImage,
    spacing: u32,
    pub(crate) char_dimensions: (u32, u32),
    alphabet: Vec<char>,

    mapping: HashMap<u32, char>,
    chars_to_idx: HashMap<char, usize>,
}

impl BitmapFont {
    pub fn open3x5(path: &str, alphabet: &str) -> BitmapFont {
        let mut bf = BitmapFont {
            image: image::open(path).unwrap(),
            spacing: 1,
            char_dimensions: (3, 5),
            alphabet: alphabet.chars().collect(),
            mapping: HashMap::new(),
            chars_to_idx: HashMap::new(),
        };

        bf.init_mapping();
        bf
    }

    fn init_mapping(&mut self) {
        let mut char_idx = 0;
        for x in (0..self.image.width()).step_by((self.char_dimensions.0 + self.spacing) as usize) {
            for y in (0..self.image.height()).step_by((self.char_dimensions.1 + self.spacing) as usize) {
                let mut key: u32 = 0;
                for xx in x..x + self.char_dimensions.0 {
                    for yy in y..y + self.char_dimensions.1 {
                        key = (key << 1);
                        if !self.image.get_pixel(xx, yy).is_blank() {
                            key = key | 1;
                        }
                    }
                }
                if key != 0 {
                    self.mapping.insert(key, self.alphabet[char_idx]);
                    self.chars_to_idx.insert(self.alphabet[char_idx], char_idx);
                }
            }
            char_idx += 1
        }
    }

    pub fn get_char(&self, image: &dyn DataImage, x: u32, y: u32) -> Option<char> {
        let mut key: u32 = 0;
        for xx in x..x + self.char_dimensions.0 {
            for yy in y..y + self.char_dimensions.1 {
                key = (key << 1);
                if !image.get_pixel(xx, yy).is_blank() {
                    key = key | 1;
                }
            }
        }
        self.mapping.get(&key).map(|c| *c)
    }

    pub fn put_string(&self, image: &mut dyn MutDataImage, x: u32, y: u32, str: &str) {
        let mut cx = x;
        for chr in str.chars() {
            if let Some(idc) = self.chars_to_idx.get(&chr) {
                let char_idx = *idc as u32;
                let font_x = char_idx * self.char_dimensions.0 + (char_idx * self.spacing);
                for dx in 0..self.char_dimensions.0 {
                    for dy in 0..self.char_dimensions.1 {
                        image.set_pixel(cx + dx, y + dy, &self.image.get_pixel(font_x + dx, dy));
                    }
                }
            }
            cx += self.char_dimensions.0 + self.spacing;
        }
    }
}

