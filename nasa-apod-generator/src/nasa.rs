use std::error::Error;

use image::DynamicImage;
use image_effects::{prelude::{Filter, Dither}, Affectable, utils::image::ImageRequest};
use palette::rgb::Rgb;
use rand::rngs::ThreadRng;
use serde_json::Value;

pub const API_URL: &'static str = "https://images-api.nasa.gov";

pub const BASE_URL: &'static str = "https://images-api.nasa.gov";

use crate::utils::generate_random_date_between;

#[inline] pub fn apod(api_key: &str, date: &str) -> String {
    format!("https://api.nasa.gov/planetary/apod?api_key={}&date={}&hd=true", api_key, date)
}

type UtilResult<T> = Result<T, Box<dyn Error>>;

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

pub struct ApodResponse {
    pub image: DynamicImage,
    pub url: String,
    pub title: String,
    pub explanation: String,
    pub date: String,
}

impl ApodResponse {
    pub fn new_from_value(value: Value, use_hd: bool, date: &str) -> UtilResult<ApodResponse> {
        
        let url = if use_hd {
            &value["hdurl"]
        } else {
            &value["url"]
        };

        let url = url.as_str().unwrap();
        let title = *&value["title"].as_str().unwrap();
        let explanation = *&value["explanation"].as_str().unwrap();

        let image = ImageRequest::Url {
            url: url,
            max_dim: None,
        }.perform()?;

        Ok(ApodResponse { 
            image: image,
            url: url.into(),
            title: title.into(),
            explanation: explanation.into(),
            date: date.into(),
        })
    }
}

pub fn get_apod_for_date(api_key: &str, date: &str, use_hd: bool) -> UtilResult<ApodResponse> {
    println!("retrieving apod at {date} from nasa...");

    let body = reqwest::blocking::get(apod(api_key, date))?.json::<serde_json::Value>()?;

    ApodResponse::new_from_value(body, use_hd, date)
}

pub fn get_random_apod(rng: &mut ThreadRng, api_key: &str, use_hd: bool) -> UtilResult<ApodResponse> {
    get_apod_for_date(
        api_key,
        &generate_random_apod_date(rng),
        use_hd
    )
}

pub fn dither_random_apod_image(rng: &mut ThreadRng, api_key: &str, palette: &(&str, Vec<Rgb>), use_hd: bool) -> UtilResult<ApodResponse> {
    let mut response = get_random_apod(rng, api_key, use_hd)?;

    response.image = response.image
        // .apply(&Filter::Contrast(1.2))
        .apply(&Dither::Bayer(8, &palette.1));
    //  .save(format!("./nasa-apod-generator/data/nasa-output-{}-({apod_date}){}.png", palette.0, if use_hd { "-hd" } else { "" }));

    Ok(response)
}