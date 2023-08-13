use std::error::Error;

use image::{GenericImageView, DynamicImage};
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
    
    enum ImageUser {
        Save,
        Cohost,
    }

    {
        let _dead_warning_mute = (ImageUser::Cohost, ImageUser::Save);
    }

    const DO_WITH_IMAGE: ImageUser = ImageUser::Save;

    let session = Session::login(&email, &password).await?;

    let mut rng = rand::thread_rng();
    for i in 0..ITERATIONS {
        let palette = palettes.choose(&mut rng).unwrap();
        println!("generating image {i} using palette [{}]...", palette.0);

        let (image, date) = dither_random_apod_image(&mut rng, &api_key, palette, USE_HD).unwrap();

        match DO_WITH_IMAGE {
            ImageUser::Save => save_image_locally(image, palette.0, &date)?,
            ImageUser::Cohost => dispatch_apod_image_to_cohost(image, &session, palette.0, date).await?,
        }        
    }

    Ok(())
}

fn save_image_locally(image: DynamicImage, palette_name: &str, date: &str) -> Result<(), Box<dyn Error>> {
    Ok(image.save_with_format(
        format!("./nasa-apod-generator/data/nasa-output-{palette_name}-{date}.png"),
        image::ImageFormat::Png
    )?)
}

async fn dispatch_apod_image_to_cohost(image: DynamicImage, session: &Session, palette_name: &str, date: String) -> Result<(), Box<dyn Error>> {
    let (width, height) = image.dimensions();
    let metadata = eggbug::MediaMetadata::Image { 
        width: Some(width),
        height: Some(height), 
    };

    let bytes = image.into_bytes();
    let attachment = Attachment::new(bytes, format!("{date}-{}", palette_name), "".into(), metadata);

    let mut post = Post {
        adult_content: false,
        headline: format!("{date}"),
        ask: None,
        attachments: vec![
            attachment
        ],
        markdown: format!("**Palette:** *{}*", palette_name),
        tags: vec![
            "apod".into(),
            "astronomy photo of the day".into(),
            "nasa".into(),
            date,
            format!("palette({})", palette_name),
        ],
        content_warnings: vec![],
        draft: false,
        metadata: None,
    };

    let _ = session.create_post("ditherpod", &mut post).await?;

    Ok(())
}