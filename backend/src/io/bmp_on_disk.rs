use std::cell::{RefCell, RefMut};
use std::fs::File;
use std::io::{Error, Read, Seek, SeekFrom, Write};
use std::ops::Div;
use log::info;

use byteorder::{LittleEndian, ReadBytesExt};
use image::{ColorType, RgbImage};
use image::codecs::png::PngEncoder;

use crate::image::{StorableImage, SyncResponse};
use crate::model::colors::RGB;
use std::fmt::{Debug, Formatter};

const SLICE_STEP: usize = 1024; //todo: settings
const KEEP_IN_MEMORY_INV: usize = 2;

#[derive(Debug, Clone)]
struct BMPParams {
    width: u32,
    height: u32,
    data_offset: u32,
    data_padding: u32,        //actually u8
}

struct BMPSlice {
    y_from: u32,
    y_to_exclusive: u32,
    data: Option<Box<[u8]>>,
    dirty: bool,

    bmp_params: BMPParams,
    loaded_nr: u32
}

impl BMPSlice {
    fn new(y_from: u32, y_to_exclusive: u32, bmp_params: BMPParams) -> Self {
        Self {
            y_from,
            y_to_exclusive,
            data: None,
            dirty: false,
            bmp_params: bmp_params.clone(),
            loaded_nr: 0
        }
    }

    fn load(&mut self, file: &mut File) {
        if self.is_loaded() { return; }

        let mut buffer = vec![0; self.capacity()];
        file.seek(self.seek()).expect("Cannot seek");
        file.read_exact(buffer.as_mut_slice()).expect("Cannot read");
        self.data = Some(buffer.into_boxed_slice());
        info!("Loaded slice {}-{} [{} bytes from {:?} {:?}]", self.y_from, self.y_to_exclusive, self.capacity(), self.seek(), self.bmp_params)
    }

    fn capacity(&self) -> usize {
        ((self.y_to_exclusive - self.y_from) * (self.bmp_params.width * 3 + self.bmp_params.data_padding)) as usize
    }

    fn seek(&self) -> SeekFrom {
        SeekFrom::Start((self.bmp_params.data_offset + self.y_from * (self.bmp_params.width * 3 + self.bmp_params.data_padding)) as u64)
    }

    fn unload(&mut self) {
        assert_eq!(self.dirty, false);
        self.data = None;
        info!("Unload slice {}-{}", self.y_from, self.y_to_exclusive)
    }

    fn save_if_loaded_and_dirty(&mut self, file: &mut File) {
        if self.data.is_some() && self.dirty {
            file.seek(self.seek()).unwrap();
            file.write(self.data.as_ref().unwrap().as_ref()).expect("Cannot write");
            file.sync_all().expect("Cannot sync"); //todo: here?
            self.dirty = false;
            info!("Saved slice {}-{}", self.y_from, self.y_to_exclusive)
        }
    }


    fn is_loaded(&self) -> bool {
        self.data.is_some()
    }

    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        assert!(self.is_loaded());
        let idx = (y - self.y_from) * (3 * self.bmp_params.width + self.bmp_params.data_padding) + x * 3;
        let idx = idx as usize;

        let data = self.data.as_ref().unwrap();
        RGB::new(data[idx + 2], data[idx + 1], data[idx])
    }

    fn set_pixel(&mut self, x: u32, y: u32, rgb: &RGB) {
        assert!(self.is_loaded());
        let idx = (y - self.y_from) * (3 * self.bmp_params.width + self.bmp_params.data_padding) + x * 3;
        let idx = idx as usize;
        let data = self.data.as_mut().unwrap().as_mut();
        //bmp is BGR
        data[idx] = rgb.b;
        data[idx + 1] = rgb.g;
        data[idx + 2] = rgb.r;
        self.dirty = true;
    }
}

pub struct BMPOnDiskImage {
    file: RefCell<File>,
    bmp_params: BMPParams,
    slices: Vec<RefCell<BMPSlice>>,
    next_loaded_nr: RefCell<u32>
}

impl BMPOnDiskImage {
    pub(crate) fn new(mut file: File) -> Self {
        //read header, create bmp params
        file.seek(SeekFrom::Start(10)).unwrap();
        let data_offset = file.read_u32::<LittleEndian>().unwrap();

        file.seek(SeekFrom::Current(4)).unwrap();
        let width = file.read_u32::<LittleEndian>().unwrap();
        let height = file.read_u32::<LittleEndian>().unwrap();
        let data_padding = (((width * 3) as f32).div(4.0).ceil() * 4.0) as u32 - (width * 3);
        let bmp_params = BMPParams { width, height, data_offset, data_padding };
        let mut slices = vec![];
        for y in (0..height).step_by(SLICE_STEP) {
            slices.push(RefCell::new(BMPSlice::new(y, (y + SLICE_STEP as u32).min(height), bmp_params.clone())))
        }
        Self {
            file: RefCell::new(file),
            bmp_params: bmp_params.clone(),
            slices,
            next_loaded_nr: RefCell::new(1)
        }
    }

    fn load_slice_if_needed(&self, slice: &mut RefMut<BMPSlice>) {
        if !slice.is_loaded() {
            slice.load(&mut self.file.borrow_mut());
            slice.loaded_nr = *self.next_loaded_nr.borrow();
            *self.next_loaded_nr.borrow_mut() += 1;
        }
    }
}

impl StorableImage for BMPOnDiskImage {
    fn get_pixel(&self, x: u32, y: u32) -> RGB {
        let y = self.bmp_params.height - y - 1;
        let idx = y as usize / SLICE_STEP;
        let mut slice = self.slices[idx].borrow_mut();
        self.load_slice_if_needed(&mut slice);
        return slice.get_pixel(x, y);
    }

    fn set_pixel(&mut self, x: u32, y: u32, rgb: &RGB) {
        let y = self.bmp_params.height - y - 1;
        let idx = y as usize / SLICE_STEP;
        let mut slice = self.slices[idx].borrow_mut();
        self.load_slice_if_needed(&mut slice);
        slice.set_pixel(x, y, rgb);
    }

    fn width(&self) -> u32 {
        self.bmp_params.width
    }

    fn height(&self) -> u32 {
        self.bmp_params.height
    }

    fn sync(&mut self) -> Result<SyncResponse, Error> {
        //todo: last modified - also check
        let mut file_ref = self.file.borrow_mut();
        let mut loaded = 0;
        let count = self.slices.len();
        for slice in &self.slices {
            let mut slice = slice.borrow_mut();
            slice.save_if_loaded_and_dirty(&mut file_ref);
            if slice.is_loaded() {
                loaded += 1;
            }
        }
        while count > 1 && loaded >= count / KEEP_IN_MEMORY_INV {
            self.slices.iter()
                .filter(|s| (*s).borrow().is_loaded())
                .min_by(|s1, s2| (*s1).borrow().loaded_nr.cmp(&(*s2).borrow().loaded_nr))
                .map(|s| s.borrow_mut().unload());
            loaded -= 1;
        }
        Ok(SyncResponse::Ok)
    }

    fn optimize(&self) {
        let mut loaded = 0;
        let count = self.slices.len();
        if count <= 1 { return }
        for slice in &self.slices {
            let slice = slice.borrow();
            if slice.is_loaded() {
                loaded += 1;
            }
        }
        while loaded >= count / KEEP_IN_MEMORY_INV {
            self.slices.iter()
                .filter(|s| (*s).borrow().is_loaded())
                .min_by(|s1, s2| (*s1).borrow().loaded_nr.cmp(&(*s2).borrow().loaded_nr))
                .map(|s| s.borrow_mut().unload());
            loaded -= 1;
        }
    }

    fn get_base64(&self, x: u32, y: u32, width: u32, height: u32) -> String {
        //not sure how to do it better...
        let mut temp_image: RgbImage = image::ImageBuffer::new(width, height);
        for xx in x..(x+width) {
            for yy in y..(y+height) {
                temp_image.put_pixel(xx - x, yy - y, self.get_pixel(xx, yy).into());
            }
        }
        let mut buf: Vec<u8> = vec![];
        PngEncoder::new(&mut buf)
            .encode(temp_image.as_raw(), temp_image.dimensions().0, temp_image.dimensions().1, ColorType::Rgb8).unwrap();
        base64::encode(&buf)
    }
}

impl Debug for BMPOnDiskImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BMPOnDiskImage")
            .field("info", &self.bmp_params)
            .finish()
    }
}