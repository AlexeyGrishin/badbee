use crate::model::model::Vector2D;
use crate::model::colors::{RGB, BLANK};
use std::fmt::Debug;

pub struct ImageView<'a> {
    image: &'a mut BoxedStorableImage,
    x: u32,
    y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl<'a> ImageView<'a> {
    pub fn new(image: &'a mut BoxedStorableImage, from: Vector2D, to_inclusive: Vector2D) -> Self {
        ImageView { image: image, x: from.x, y: from.y, width: to_inclusive.x - from.x + 1, height: to_inclusive.y - from.y + 1 }
    }

    pub fn from(img: &mut BoxedStorableImage) -> ImageView {
        let size = Vector2D::new(img.width() - 1, img.height() - 1);
        ImageView::new(img, Vector2D::new(0, 0), size)
    }

    pub(crate) fn get_pixel(&self, x: u32, y: u32) -> RGB {
        if x >= self.width || y >= self.height {
            BLANK
        } else {
            self.image.get_pixel(self.x + x, self.y + y)
        }
    }

    pub fn set_pixel<T>(&mut self, x: u32, y: u32, rgb: T) -> Result<(), ()> where T: Into<RGB> {
        if x < self.width && y < self.height {
            self.image.as_mut().set_pixel(self.x + x, self.y + y, &rgb.into());
            Result::Ok(())
        } else {
            Result::Err(())
        }
    }

    pub fn fill(&mut self, rgb: &RGB) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.set_pixel(x, y, *rgb).unwrap();
            }
        }
    }
    pub fn clear(&mut self) {
        self.fill(&RGB::new(255, 255, 255))
    }

    pub fn optimize(&self) {
        self.image.optimize();
    }

    pub fn get_base64(&self) -> String {
        self.image.get_base64(self.x, self.y, self.width, self.height)
    }
}


#[derive(Debug)]
pub enum SyncResponse {
    Ok,
    Reloaded,
}


pub trait StorableImage: Debug {
    fn get_pixel(&self, x: u32, y: u32) -> RGB;

    fn set_pixel(&mut self, x: u32, y: u32, rgb: &RGB);

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn sync(&mut self) -> Result<SyncResponse, std::io::Error>;

    fn optimize(&self);

    fn get_base64(&self, x: u32, y: u32, width: u32, height: u32) -> String;
}

pub type BoxedStorableImage = Box<dyn StorableImage + Send>;