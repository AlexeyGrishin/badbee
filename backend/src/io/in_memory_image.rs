use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use image::{ColorType, DynamicImage, GenericImage, GenericImageView, ImageResult};
use image::codecs::png::PngEncoder;

use crate::image::{StorableImage, SyncResponse};
use crate::model::colors::RGB;
use std::fmt::{Debug, Formatter};

pub struct InMemoryImage {
    image: DynamicImage,
    path: PathBuf,
    dirty: bool,
    last_modified_time: SystemTime,
}

impl InMemoryImage {
    pub(crate) fn new(path: &Path) -> Self {
        Self {
            image: image::open(path).unwrap(),
            path: path.to_path_buf(),
            dirty: false,
            last_modified_time: std::fs::metadata(path).unwrap().modified().unwrap(),
        }
    }
}

impl StorableImage for InMemoryImage {
    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        self.image.get_pixel(x, y).into()
    }


    fn set_pixel(&mut self, x: u32, y: u32, rgb: &RGB) {
        self.image.put_pixel(x, y, rgb.into());
        self.dirty = true;
    }

    fn width(&self) -> u32 {
        self.image.width()
    }

    fn height(&self) -> u32 {
        self.image.height()
    }

    fn sync(&mut self) -> Result<crate::image::SyncResponse, Error> {
        let path = self.path.as_path();
        let modified = std::fs::metadata(path)?.modified().unwrap();
        if self.dirty {
            let copied = self.image.clone();
            let copied_path = path.to_path_buf();
            tokio::spawn( async move {
                if let ImageResult::Err(e) = copied.save(copied_path) {
                    log::error!("Error during saving {}", e)
                }
            });
            self.dirty = false;
            Ok(SyncResponse::Ok)
        } else if modified > self.last_modified_time {
            self.last_modified_time = modified;
            self.image = image::open(path).map_err(|_| Error::new(ErrorKind::Other, "Cannot load"))?;
            Ok(SyncResponse::Reloaded)
        } else {
            Ok(SyncResponse::Ok)
        }
    }

    fn optimize(&self) {

    }

    fn get_base64(&self, x: u32, y: u32, width: u32, height: u32) -> String {
        let sub_image = self.image.view(x, y, width, height);
        let mut buf: Vec<u8> = vec![];
        PngEncoder::new(&mut buf)
            .encode(sub_image.to_image().as_raw(), sub_image.dimensions().0, sub_image.dimensions().1, ColorType::Rgba8).unwrap();
        base64::encode(&buf)
    }
}


impl Debug for InMemoryImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryImage")
            .field("path", &self.path)
            .finish()
    }
}