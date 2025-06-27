use std::{error::Error, fs::File, io::Read};

use image::{
    codecs::gif::GifDecoder, imageops, io::Reader, AnimationDecoder, DynamicImage, Frame,
    GenericImageView,
};

#[derive(Clone)]
pub enum ImageResult {
    Image(DynamicImage),
    Gif(Vec<Frame>),
}

#[derive(Debug)]
pub struct Empty;

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

#[derive(Debug, Clone)]
pub enum SourceKind {
    Url(String),
    File(String),
}

#[derive(Debug, Clone)]
pub enum MediaType {
    Image,
    Gif,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub source: SourceKind,
    pub media_type: MediaType,
    pub max_dim: Option<usize>,
}

type UtilResult<T> = Result<T, Box<dyn Error>>;

impl Source {
    pub fn perform(&self) -> UtilResult<ImageResult> {
        let result = match (&self.media_type, &self.source, &self.max_dim) {
            (MediaType::Image, SourceKind::File(target), None) => {
                load_image_from_path(target)?.into()
            }
            (MediaType::Image, SourceKind::File(target), Some(max_dim)) => {
                load_image_from_path_with_max_dim(target, *max_dim)?.into()
            }
            (MediaType::Image, SourceKind::Url(target), None) => {
                load_image_from_url(target)?.into()
            }
            (MediaType::Image, SourceKind::Url(target), Some(max_dim)) => {
                load_image_from_url_with_max_dim(target, *max_dim)?.into()
            }
            (MediaType::Gif, SourceKind::File(target), _) => load_gif_from_file(target)?.into(),
            (MediaType::Gif, SourceKind::Url(target), _) => load_gif_from_url(target)?.into(),
        };

        Ok(result)
    }
}

// resizers
pub fn resize_image(image: &DynamicImage, factor: f32) -> DynamicImage {
    let (x, y) = image.dimensions();
    let mul = |int: u32, float: f32| (int as f32 * float) as u32;
    image.resize(mul(x, factor), mul(y, factor), imageops::Nearest)
}

pub fn resize_image_with_max_dim(image: &DynamicImage, maxdim: usize) -> DynamicImage {
    let (x, y) = image.dimensions();
    if maxdim < x.max(y) as usize {
        resize_image(image, maxdim as f32 / x.max(y) as f32)
    } else {
        image.clone()
    }
}

// loaders

// image

fn load_image_from_path(path: &str) -> UtilResult<DynamicImage> {
    Ok(Reader::open(path)?.decode()?)
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

    Ok(GifDecoder::new(data.as_slice())?
        .into_frames()
        .collect_frames()?)
}
