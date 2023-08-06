use image_filters::utils::ImageFilterResult;
use rand::prelude::SliceRandom;
use dotenv::dotenv;

pub mod nasa;
pub mod base;
pub mod gradients;

use crate::{nasa::dither_apod, gradients::palettes};

const ITERATIONS: usize = 3;

fn main() -> ImageFilterResult<()> {
    dotenv().ok();
    let palettes = palettes();
    let api_key = std::env::var("NASA_API_KEY").expect("NASA_API_KEY must be set in the environment/.env.");

    let mut rng = rand::thread_rng();
    for i in 0..ITERATIONS {
        let palette = palettes.choose(&mut rng).unwrap();
        println!("generating image {i} using palette [{}]...", palette.0);
        if let Err(error) = dither_apod(&api_key, palette) {
            println!("error when dithering apod: {:?}", error);
        };
    }

    Ok(())
}
