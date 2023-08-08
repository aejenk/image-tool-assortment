use image::DynamicImage;
use image_filters::{utils::{image::load_image_from_url_with_max_dim, Error}, AdjustableImage, prelude::{Filter, Dither}};
use palette::rgb::Rgb;
use rand::rngs::ThreadRng;

pub const API_URL: &'static str = "https://images-api.nasa.gov";

pub const BASE_URL: &'static str = "https://images-api.nasa.gov";

use crate::utils::generate_random_date_between;

#[inline] pub fn apod(api_key: &str, date: &str) -> String {
    format!("https://api.nasa.gov/planetary/apod?api_key={}&date={}&hd=true", api_key, date)
}

#[derive(Debug)]
pub enum NasaError {
    Reqwest(reqwest::Error),
    Image(Error),
    NoUrl,
}

impl From<reqwest::Error> for NasaError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<Error> for NasaError {
    fn from(value: Error) -> Self {
        Self::Image(value)
    }
}

type NasaResult<T> = Result<T, NasaError>;

pub fn generate_random_apod_date(rng: &mut ThreadRng) -> String {
    // the first APOD image was in 1995/06/16.
    // as for the end date, it's hardcoded but technically should default to today.
    // i just didn't wanna look into getting that date yet - maybe soon
    let (year, month, day) = generate_random_date_between(
        rng,
        (1995, 06, 16),
        (2023, 08, 06)
    );

    format!("{year}-{month:0>2}-{day:0>2}")
}

pub fn get_apod_for_date(api_key: &str, date: &str, use_hd: bool) -> NasaResult<DynamicImage> {
    println!("retrieving apod at {date} from nasa...");

    let body = reqwest::blocking::get(apod(api_key, date))?.json::<serde_json::Value>()?;

    let url = if use_hd {
        &body["hdurl"]
    } else {
        &body["url"]
    };

    let maxdim = if use_hd { 2160 } else { 1080 };

    if let serde_json::Value::String(url) = url {
        Ok(load_image_from_url_with_max_dim(url, maxdim)?)
    } else {
        Err(NasaError::NoUrl)
    }
}

pub fn dither_random_apod_image(rng: &mut ThreadRng, api_key: &str, palette: &(&str, Vec<Rgb>), use_hd: bool) -> NasaResult<DynamicImage> {
    let apod_date = generate_random_apod_date(rng);
    let image = get_apod_for_date(api_key, &apod_date, use_hd)?;

    let _ = image.clone()
     .apply(Filter::Contrast(1.2))
     .apply(Dither::Bayer(8, &palette.1))
     .save(format!("./nasa-apod-generator/data/nasa-output-{}-({apod_date}){}.png", palette.0, if use_hd { "-hd" } else { "" }));

    Ok(image)
}