use image_filters::utils::ImageFilterResult;
use rand::prelude::SliceRandom;

pub mod nasa;
pub mod base;
pub mod gradients;

use crate::{nasa::dither_apod, gradients::palettes};

const ITERATIONS: usize = 10;

fn main() -> ImageFilterResult<()> {
    let palettes = palettes();

    let mut rng = rand::thread_rng();
    for i in 0..ITERATIONS {
        let palette = palettes.choose(&mut rng).unwrap();
        println!("generating image {i} using palette [{}]...", palette.0);
        if let Err(error) = dither_apod(palette) {
            println!("error when dithering apod: {:?}", error);
        };
    }

    Ok(())
}
