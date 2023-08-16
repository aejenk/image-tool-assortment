use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read};

use base64::Engine;
use image::codecs::gif::GifDecoder;
use image::io::Reader as ImageReader;
use image::{self, imageops, DynamicImage, GenericImageView, Frame, AnimationDecoder};

type UtilResult<T> = Result<T,Box<dyn Error>>;

pub enum ImageRequest<'a> {
    Url {
        url: &'a str,
        max_dim: Option<usize>,
    },
    File{
        file: &'a str,
        max_dim: Option<usize>,
    },
}

impl<'a> ImageRequest<'a> {
    pub fn perform(&self) -> UtilResult<DynamicImage> {
        match self {
            ImageRequest::File {
                file,
                max_dim 
            } => if let Some(max_dim) = max_dim {
                load_image_from_path_with_max_dim(file,*max_dim as u32)
            } else {
                load_image_from_path(file)
            },
            ImageRequest::Url {
                url,
                max_dim
            } => if let Some(max_dim) = max_dim {
                load_image_from_url_with_max_dim(url,*max_dim as u32)
            } else {
                load_image_from_url(url)
            }
        }
    }
}

pub enum GifRequest<'a> {
    Url {
        url: &'a str,
        // max_dim: Option<usize>,
    },
    File{
        file: &'a str,
        // max_dim: Option<usize>,
    },
}

impl<'a> GifRequest<'a> {
    pub fn perform(&self) -> UtilResult<Vec<Frame>> {
        match self {
            GifRequest::File {
                file,
                // max_dim 
            } => load_gif_from_file(file),
            GifRequest::Url {
                url,
                // max_dim
            } => load_gif_from_url(url),
        }
    }
}

pub fn image_to_b64(image: &DynamicImage) -> UtilResult<String> {
    let mut buf = Vec::new();
    image.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(buf))
}

pub fn b64_to_image(b64: &str) -> UtilResult<DynamicImage> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
    Ok(image::load_from_memory(&bytes)?)
}

// Resize functions

pub fn resize_image(image: &DynamicImage, factor: f32) -> DynamicImage {
    let (x, y) = image.dimensions();
    let mul = |int: u32, float: f32| (int as f32 * float) as u32;
    image.resize(mul(x, factor), mul(y, factor), imageops::Nearest)
}

pub fn resize_image_with_max_dim(image: &DynamicImage, maxdim: u32) -> DynamicImage {
    let (x, y) = image.dimensions();
    if maxdim < x.max(y) {
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

fn load_image_from_path_with_max_dim(path: &str, maxdim: u32) -> UtilResult<DynamicImage> {
    let image = load_image_from_path(path)?;
    Ok(resize_image_with_max_dim(&image, maxdim))
}

fn load_image_from_url(url: &str) -> UtilResult<DynamicImage> {
    let img_bytes = reqwest::blocking::get(url)?.bytes()?;
    Ok(image::load_from_memory(&img_bytes)?)
}

fn load_image_from_url_with_max_dim(url: &str, maxdim: u32) -> UtilResult<DynamicImage> {
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