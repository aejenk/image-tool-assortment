use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read};

use base64::Engine;
use image::codecs::gif::GifDecoder;
use image::io::Reader as ImageReader;
use image::{self, imageops, DynamicImage, GenericImageView, Frame, AnimationDecoder, Frames, EncodableLayout};

type UtilResult<T> = Result<T,Box<dyn Error>>;

#[derive(Clone, Copy)]
pub enum PathType { Url, File }

#[derive(Clone, Copy)]
pub enum FileType { Image, Gif }

pub struct ImageRequest {
    path_type: PathType,
    file_type: FileType,
    target: String,
    max_dim: Option<usize>,
}

pub enum ImageResult {
    Image(DynamicImage),
    Gif(Vec<Frame>),
}

impl From<DynamicImage> for ImageResult {
    fn from(value: DynamicImage) -> Self {
        Self::Image(value)
    }
}

impl From<Vec<Frame>> for ImageResult {
    fn from(value: Vec<Frame>) -> Self {
        Self::Gif(value)
    }
}

#[derive(Debug)]
pub struct Empty;

impl std::fmt::Display for Empty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl Error for Empty {}

impl ImageResult {
    pub fn into_image(self) -> Result<DynamicImage, Empty> {
        if let ImageResult::Image(image) = self {
            Ok(image)
        } else {
            Err(Empty)
        }
    }

    pub fn into_gif(self) -> Result<Vec<Frame>, Empty> {
        if let ImageResult::Gif(gif) = self {
            Ok(gif)
        } else {
            Err(Empty)
        }
    }
}

impl ImageRequest {
    pub fn new(target: String) -> Self {
        Self { path_type: PathType::File, file_type: FileType::Image, target, max_dim: None }
    }

    pub fn url(mut self) -> Self {
        self.path_type = PathType::Url;
        self
    }

    pub fn file(mut self) -> Self {
        self.path_type = PathType::File;
        self
    }

    pub fn image(mut self) -> Self {
        self.file_type = FileType::Image;
        self
    }

    pub fn gif(mut self) -> Self {
        self.file_type = FileType::Gif;
        self
    }

    pub fn with_max_dim(mut self, max_dim: usize) -> Self {
        self.max_dim = Some(max_dim);
        self
    }

    pub fn keep_size(mut self) -> Self {
        self.max_dim = None;
        self
    }

    pub fn perform(&self) -> UtilResult<ImageResult> {
        let result = match (self.file_type, self.path_type, self.max_dim) {
            (FileType::Image, PathType::File, None) => load_image_from_path(&self.target)?.into(),
            (FileType::Image, PathType::File, Some(max_dim)) => load_image_from_path_with_max_dim(&self.target, max_dim)?.into(),
            (FileType::Image, PathType::Url, None) => load_image_from_url(&self.target)?.into(),
            (FileType::Image, PathType::Url, Some(max_dim)) => load_image_from_url_with_max_dim(&self.target, max_dim)?.into(),
            (FileType::Gif, PathType::File, _) => load_gif_from_file(&self.target)?.into(),
            (FileType::Gif, PathType::Url, _) => load_gif_from_url(&self.target)?.into(),
        };

        Ok(result)
    }
}

pub fn image_to_b64(image: &DynamicImage) -> UtilResult<String> {
    let mut buf = Vec::new();
    image.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(buf))
}

pub fn gif_to_b64(gif: Vec<Frame>) -> UtilResult<String> {    
    let bytes = gif
        .into_iter()
        .map(|frame| frame.buffer().as_bytes().to_vec())
        .collect::<Vec<_>>().concat();

    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes))
}

pub fn b64_to_image(b64: &str) -> UtilResult<DynamicImage> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
    Ok(image::load_from_memory(&bytes)?)
}

pub fn b64_to_gif(b64: &str) -> UtilResult<Vec<Frame>> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
    Ok(GifDecoder::new(bytes.as_slice())?.into_frames().collect_frames()?)
}

// Resize functions

pub fn resize_image(image: &DynamicImage, factor: f32) -> DynamicImage {
    let (x, y) = image.dimensions();
    let mul = |int: u32, float: f32| (int as f32 * float) as u32;
    image.resize(mul(x, factor), mul(y, factor), imageops::Nearest)
}

pub fn resize_image_with_max_dim(image: &DynamicImage, maxdim: usize) -> DynamicImage {
    let (x, y) = image.dimensions();
    if maxdim < x.max(y) as usize {
        resize_image(&image, maxdim as f32 / x.max(y) as f32)
    } else {
        image.clone()
    }
}

// Loader functions

// image

fn load_image_from_path(path: &str) -> UtilResult<DynamicImage> {
    Ok(ImageReader::open(path)?.decode()?)
}

fn load_image_from_path_with_max_dim(path: &str, maxdim: usize) -> UtilResult<DynamicImage> {
    let image = load_image_from_path(path)?;
    Ok(resize_image_with_max_dim(&image, maxdim))
}

fn load_image_from_url(url: &str) -> UtilResult<DynamicImage> {
    let img_bytes = reqwest::blocking::get(url)?.bytes()?;
    Ok(image::load_from_memory(&img_bytes)?)
}

fn load_image_from_url_with_max_dim(url: &str, maxdim: usize) -> UtilResult<DynamicImage> {
    let image = load_image_from_url(url)?;
    Ok(resize_image_with_max_dim(&image, maxdim))
}

// gif

fn load_gif_from_file(path: &str) -> UtilResult<Vec<Frame>> {
    let file = File::open(path).unwrap();
    Ok(GifDecoder::new(file)?.into_frames().collect_frames()?)
}

fn load_gif_from_url(url: &str) -> UtilResult<Vec<Frame>> {
    let mut gif_bytes = reqwest::blocking::get(url)?;
    
    let mut data = Vec::new();
    gif_bytes.read_to_end(&mut data)?;
    
    Ok(GifDecoder::new(data.as_slice())?.into_frames().collect_frames()?)
}