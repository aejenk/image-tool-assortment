use std::{marker::PhantomData, iter::Map};

use image::DynamicImage;
use image_filters::{utils::{image::{load_image_from_url_with_max_dim, load_image_from_url}, ImageFilterResult, Error}, AdjustableImage, prelude::{Filter, Dither}};
use palette::rgb::Rgb;
use serde::{Serialize, Deserialize};

pub const API_URL: &'static str = "https://images-api.nasa.gov";

pub const BASE_URL: &'static str = "https://images-api.nasa.gov";

use rand::distributions::{Distribution, Uniform};

#[inline] pub fn search() -> String {
    format!("{}/search", BASE_URL)
}

#[inline] pub fn asset(nasa_id: &str) -> String {
    format!("{}/asset/{}", BASE_URL, nasa_id)
}

#[inline] pub fn metadata(nasa_id: &str) -> String {
    format!("{}/metadata/{}", BASE_URL, nasa_id)
}

#[inline] pub fn captions(nasa_id: &str) -> String {
    format!("{}/captions/{}", BASE_URL, nasa_id)
}

#[inline] pub fn album(album_name: &str) -> String {
    format!("{}/album/{}", BASE_URL, album_name)
}

#[inline] pub fn apod(api_key: &str, date: &str) -> String {
    format!("https://api.nasa.gov/planetary/apod?api_key={}&date={}", api_key, date)
}

pub enum MediaType {
    IMAGE, VIDEO, AUDIO
}

#[derive(Default)]
pub struct SearchQueries {
    pub q: Option<String>,
    pub center: Option<String>,
    pub description: Option<String>,
    pub description_508: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub location: Option<String>,
    pub media_type: Option<MediaType>,
    pub nasa_id: Option<String>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub photographer: Option<String>,
    pub secondary_creator: Option<String>,
    pub title: Option<String>,
    pub year_start: Option<String>,
    pub year_end: Option<String>,
}

pub enum Endpoints {
    Search(SearchQueries)
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

pub fn generate_random_apod_date() -> String {
    let mut rng = rand::thread_rng();
    // the first image was set on 1995-06-16

    let earliest = (1995, 06, 16);
    let latest = (2023, 08, 06);

    let year = Uniform::from(earliest.0..latest.0);
    let month = Uniform::from(1..=12);
    let day = Uniform::from(1..=31);

    let (year, mut month, mut day) = (
        year.sample(&mut rng),
        month.sample(&mut rng),
        day.sample(&mut rng),
    );

    if year == earliest.0 {
        month = month.max(earliest.1);
        day = day.max(earliest.2);
    } else if year == latest.0 {
        month = month.min(latest.1);
        day = day.min(latest.2);
    }

    day = if month == 2 {
        if year % 4 == 0 {
            day.min(29)
        } else {
            day.min(28)
        }
    } else if (month <= 7 && month % 2 == 1) || (month > 7 && month % 2 == 0) {
        day.min(31)
    } else {
        day.min(30)
    };

    format!("{year}-{month:0>2}-{day:0>2}")
}

pub fn get_apod_for_date(api_key: &str, date: &str) -> NasaResult<DynamicImage> {
    println!("retrieving apod at {date} from nasa...");
    let mut queries = SearchQueries::default();
    queries.page = Some(1);
    queries.page_size = Some(100);

    // let body = reqwest::blocking::get(format!("{}?page=1&page_size=100&q=space", search()))?.json::<serde_json::Value>()?;

    let body = reqwest::blocking::get(apod(api_key, date))?.json::<serde_json::Value>()?;

    let url = &body["url"];

    if let serde_json::Value::String(url) = url {
        Ok(load_image_from_url_with_max_dim(url, 1080)?)
    } else {
        Err(NasaError::NoUrl)
    }
}

pub fn dither_apod(api_key: &str, palette: &(&str, Vec<Rgb>)) -> NasaResult<DynamicImage> {
    let apod_date = generate_random_apod_date();
    let image = get_apod_for_date(api_key, &apod_date)?;

    let _ = image.clone()
     .apply(Filter::Contrast(1.2))
     .apply(Dither::Bayer(8, &palette.1))
     .save(format!("./nasa-apod-generator/data/nasa-output-{}-({apod_date}).png", palette.0));

    Ok(image)
}