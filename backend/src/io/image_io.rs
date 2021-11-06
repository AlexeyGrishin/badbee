use std::fs::{OpenOptions};
use std::path::{Path};
use crate::image::{BoxedStorableImage};
use crate::io::bmp_on_disk::BMPOnDiskImage;
use crate::io::in_memory_image::InMemoryImage;

pub fn load_image(path: &str) -> BoxedStorableImage {
    let path = Path::new(path);
    let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
    let extension = path.extension().unwrap().to_str().unwrap();
    match extension {
        "bmp" => Box::new(BMPOnDiskImage::new(file)),
        _ => Box::new(InMemoryImage::new(&path))
    }
}
