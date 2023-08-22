use image::DynamicImage;
use rand::Rng;
use serde_json::Value;
use common_utils::image::ImageRequest;

use crate::utils::{generate_random_date_between, NoneError, UtilResult};

#[inline] pub fn apod(api_key: &str, date: &str) -> String {
    format!("https://api.nasa.gov/planetary/apod?api_key={}&date={}&hd=true", api_key, date)
}

pub fn generate_random_apod_date(rng: &mut impl Rng) -> String {
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

        let url = url.as_str().ok_or(NoneError)?;
        let title = *&value["title"].as_str().unwrap_or("(no title)");
        let explanation = *&value["explanation"].as_str().unwrap_or("(no explanation)");

        let image = ImageRequest::new(url.into())
            .url()
            .image()
            .keep_size()
            .perform()?
            .into_image()?;

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

#[inline] pub fn get_random_apod(rng: &mut impl Rng, api_key: &str, use_hd: bool) -> UtilResult<ApodResponse> {
    get_apod_for_date(
        api_key,
        &generate_random_apod_date(rng),
        use_hd
    )
}