use std::error::Error;

use image::GenericImageView;
use rand::prelude::SliceRandom;
use dotenv::dotenv;
use eggbug::{Session, Post, Attachment};

pub mod nasa;
pub mod palettes;
mod utils;

use crate::{nasa::dither_random_apod_image, palettes::palettes};

const ITERATIONS: usize = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let palettes = palettes();
    let api_key = std::env::var("NASA_API_KEY").expect("NASA_API_KEY must be set in the environment/.env.");
    let email = std::env::var("COHOST_EMAIL").expect("COHOST_EMAIL must be set in the environment/.env.");
    let password = std::env::var("COHOST_PASSWORD").expect("COHOST_PASSWORD must be set in the environment/.env.");

    const USE_HD: bool = true;

    let session = Session::login(&email, &password).await?;

    let mut rng = rand::thread_rng();
    for i in 0..ITERATIONS {
        let palette = palettes.choose(&mut rng).unwrap();
        println!("generating image {i} using palette [{}]...", palette.0);

        let (image, date) = dither_random_apod_image(&mut rng, &api_key, palette, USE_HD).unwrap();

        let (width, height) = image.dimensions();
        let metadata = eggbug::MediaMetadata::Image { 
            width: Some(width),
            height: Some(height), 
        };

        let bytes = image.into_bytes();
        let attachment = Attachment::new(bytes, format!("{date}-{}", palette.0), "".into(), metadata);

        let mut post = Post {
            adult_content: false,
            headline: format!("{date}"),
            ask: None,
            attachments: vec![
                attachment
            ],
            markdown: format!("**Palette:** *{}*", palette.0),
            tags: vec![
                "apod".into(),
                "astronomy photo of the day".into(),
                "nasa".into(),
                date,
                format!("palette({})", palette.0),
            ],
            content_warnings: vec![],
            draft: false,
            metadata: None,
        };

        let _ = session.create_post("ditherpod", &mut post).await?;
    }

    Ok(())
}