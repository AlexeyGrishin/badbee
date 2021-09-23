extern crate image;
extern crate base64;

use std::fs;
use std::ops::Index;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::model::datatypes::flood::FLOOD_TYPE;
use crate::model::model::{DataImage, DataProvider, Field, IndexedDataImage, Model, Record, Vector, MutDataImage};
use crate::model::model_reader::load_model_into;

use self::image::{Rgba, RgbaImage, EncodableLayout, ColorType, GenericImageView, GenericImage};
use self::image::png::PngEncoder;

pub struct FileInMemory {
    path: String,
    image: RgbaImage,

    last_modified: SystemTime,

    cached_model: Mutex<Option<Model>>,     //todo: why Mutex?

    dirty: bool,
}

impl FileInMemory {
    pub(crate) fn new(path: String) -> FileInMemory {
        return FileInMemory {
            path: path.clone(),
            image: image::open(path.as_str()).expect("Something again wrong with file").to_rgba8(),
            last_modified: SystemTime::UNIX_EPOCH,
            cached_model: Mutex::new(None),
            dirty: false
        };
    }

    pub(crate) fn get_model(&self) -> &Mutex<Option<Model>> {
        &self.cached_model
    }

    pub(crate) fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn sync(&mut self) {
        if self.dirty {
            self.dirty = false;
            self.image.save(&self.path);
            println!("RESAVE");
        }
        let modified = fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .expect("Something is wrong with file")
            ;
        if modified > self.last_modified {
            println!("RELOAD");
            {
                let mut model = self.cached_model.lock().unwrap();
                *model = None;
            }

            self.image = image::open(&self.path).expect("Something again wrong with file").to_rgba8();
            let new_model: Model = self.do_load_model();
            {
                let mut model = self.cached_model.lock().unwrap();
                *model = Some(new_model)
            }
            self.last_modified = modified
        }
    }

    fn do_load_model(&self) -> Model {
        let mut model = Model::new();
        load_model_into(&mut model, self);
        return model;
    }
}

impl DataImage for FileInMemory {
    fn width(&self) -> u32 {
        self.image.width()
    }

    fn height(&self) -> u32 {
        self.image.height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> &Rgba<u8> {
        if x >= self.image.width() || y >= self.image.height() {
            return &EMPTY;
        }
        self.image.get_pixel(x, y)
    }

    fn get_base64(&self) -> String {
        let mut buf: Vec<u8> = vec![];
        PngEncoder::new(&mut buf)
            .encode(self.image.as_raw(), self.image.width(), self.image.height(), ColorType::Rgba8)
            .unwrap();
        base64::encode(&buf)
    }
}

impl Index<(u32, u32)> for FileInMemory {
    type Output = Rgba<u8>;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        return self.get_pixel(index.0, index.1);
    }
}

impl IndexedDataImage for FileInMemory {} //that strange


const EMPTY: Rgba<u8> = Rgba([0, 0, 0, 0]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

impl DataProvider for FileInMemory {
    fn load(&self, from: &Vector, to: &Vector) -> Box<dyn DataImage + '_> {
        Box::new(DataImageInMemory {
            image: &self.image,
            from: from.clone(),
            to: to.clone(),
        })
    }

    fn load_mut(&mut self, from: &Vector, to: &Vector) -> Box<dyn MutDataImage + '_> {
        Box::new(MutDataImageInMemory {
            image: &mut self.image,
            from: from.clone(),
            to: to.clone(),
        })
    }
}

struct DataImageInMemory<'a> {
    image: &'a RgbaImage,
    from: Vector,
    to: Vector,
}

impl Index<(u32, u32)> for DataImageInMemory<'_> {
    type Output = Rgba<u8>;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        return self.get_pixel(index.0, index.1);
    }
}

struct MutDataImageInMemory<'a> {
    image: &'a mut RgbaImage,
    from: Vector,
    to: Vector,
}

impl MutDataImage for MutDataImageInMemory<'_>  {
    fn set_pixel(&mut self, x: u32, y: u32, color: &Rgba<u8>) {
        self.image.put_pixel(self.from.x + x, self.from.y + y, *color);
    }

    fn width(&self) -> u32 {
        self.to.x - self.from.x + 1
    }

    fn height(&self) -> u32 {
        self.to.y - self.from.y + 1
    }

    fn clear(&mut self) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                self.set_pixel(x, y, &WHITE)
            }
        }
    }
}

impl DataImage for DataImageInMemory<'_>  {
    fn width(&self) -> u32 {
        self.to.x - self.from.x + 1
    }

    fn height(&self) -> u32 {
        self.to.y - self.from.y + 1
    }

    fn get_pixel(&self, x: u32, y: u32) -> &Rgba<u8> {
        if x >= self.image.width() || y >= self.image.height() {
            return &EMPTY;
        }
        self.image.get_pixel(self.from.x + x, self.from.y + y)
    }

    fn get_base64(&self) -> String {
        let mut buf: Vec<u8> = vec![];
        let sub_image = self.image.view(self.from.x, self.from.y, self.width(), self.height());
        PngEncoder::new(&mut buf)
            .encode(sub_image.to_image().as_raw(), sub_image.dimensions().0, sub_image.dimensions().1, ColorType::Rgba8).unwrap();
        base64::encode(&buf)
    }
}
