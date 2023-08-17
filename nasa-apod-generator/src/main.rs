use std::{error::Error, time::Duration};

use clokwerk::{AsyncScheduler, TimeUnits, Job};
use common_utils::{palette::{palettes, generate_palette_html}, image::resize_image_with_max_dim};
use image::GenericImageView;
use image_effects::{prelude::Dither, Affectable};
use nasa::ApodResponse;
use palette::rgb::Rgb;
use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};
use dotenv::dotenv;
use eggbug::{Session, Post, Attachment};
use futures::executor;
use chrono;


pub mod nasa;
mod utils;

use crate::nasa::get_random_apod;

const ITERATIONS: usize = 1;

#[tokio::main]
async fn main() {
    let mut scheduler = AsyncScheduler::new();

    println!("----- Launched on: {:?}", chrono::offset::Local::now());

    scheduler.every(1.hour()).run(|| async {
        println!("----- Executing on: {:?}", chrono::offset::Local::now());
        loop {
            if let Err(error) = execute().await {
                eprintln!("Error encountered: {}", error.to_string());
                eprintln!("Retrying...");
                continue;
            }
            println!("Completed loop! Returning to scheduler...\n");
            break;
        };
    });

    loop {
        scheduler.run_pending().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn execute() -> Result<(), Box<dyn Error>> {
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

    const DO_WITH_IMAGE: ImageUser = ImageUser::Cohost;

    let session = Session::login(&email, &password).await?;

    let mut rng = StdRng::from_entropy();
    for i in 0..ITERATIONS {
        let palette = palettes.choose(&mut rng).unwrap();
        println!("generating image {i} using palette [{}]...", palette.0);

        let mut response = get_random_apod(&mut rng, &api_key, USE_HD)?;

        response.image = resize_image_with_max_dim(&response.image, 720);

        response.image = response.image
            .apply(&Dither::Bayer(8, &palette.1));

        match DO_WITH_IMAGE {
            ImageUser::Save => save_image_locally(response, palette.0)?,
            ImageUser::Cohost => dispatch_apod_image_to_cohost(response, &session, palette.0, palette.1.clone()).await?,
        }        
    }

    Ok(())
}

fn save_image_locally(response: ApodResponse, palette_name: &str) -> Result<(), Box<dyn Error>> {
    Ok(response.image.save_with_format(
        format!("./nasa-apod-generator/data/nasa-output-{palette_name}-{}.png", response.date),
        image::ImageFormat::Png
    )?)
}

async fn dispatch_apod_image_to_cohost(mut response: ApodResponse, session: &Session, palette_name: &str, palette: Vec<Rgb>) -> Result<(), Box<dyn Error>> {
    let (mut width, mut height) = response.image.dimensions();

    if (width * height) > (1920 * 1080) {
        response.image = common_utils::image::resize_image_with_max_dim(&response.image, 720);
        (width, height) = response.image.dimensions();
    }

    let temp_file_path = "._temporary.nasa.apod.result.png";
    response.image.save(temp_file_path)?;

    let metadata = eggbug::MediaMetadata::Image { 
        width: Some(width),
        height: Some(height), 
    };

    // let bytes = image.into_bytes();
    // let mut attachment = Attachment::new(bytes, format!("{date}-{palette_name}.png"), "image/png".into(), metadata);
    let mut attachment = Attachment::new_from_file(temp_file_path, "image/png".into(), Some(metadata)).await?;
    attachment.alt_text = Some(format!("Astronomy Photo Of the Day for: {}, dithered using {palette_name}. Titled: {}", response.date, response.title));

    let palette_html = generate_palette_html(palette);

    let mut post = Post {
        adult_content: false,
        headline: format!("{} - {}", response.date, response.title),
        ask: None,
        attachments: vec![
            attachment
        ],
        markdown: format!("**Explanation:** {} <hr/>\n**Palette:** *{palette_name}*\n{palette_html}", response.explanation),
        tags: vec![
            "apod".into(),
            "astronomy photo of the day".into(),
            "nasa".into(),
            "dithering".into(),
            "dither".into(),
            response.date,
            format!("palette({})", palette_name),
        ],
        content_warnings: vec![],
        draft: false,
        metadata: None,
    };

    let _ = session.create_post("ditherpod", &mut post).await?;

    Ok(())
}