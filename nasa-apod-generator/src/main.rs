use std::{error::Error, time::Duration};

use image::{GenericImageView, DynamicImage, imageops};
use palette::rgb::Rgb;
use palettes::generate_palette_html;
use rand::prelude::SliceRandom;
use dotenv::dotenv;
use eggbug::{Session, Post, Attachment};
use job_scheduler::{Job, JobScheduler};
use futures::executor;
use chrono;

pub mod nasa;
pub mod palettes;
mod utils;

use crate::{nasa::dither_random_apod_image, palettes::palettes};

const ITERATIONS: usize = 1;

#[tokio::main]
async fn main() {
    let mut scheduler = JobScheduler::new();

    println!("----- Launched on: {:?}", chrono::offset::Local::now());

    scheduler.add(Job::new("0 0 * * * * *".parse().unwrap(), || {
        println!("----- Executing on: {:?}", chrono::offset::Local::now());
        let executor = async {
            loop {
                if let Err(error) = execute().await {
                    eprintln!("Error encountered: {}", error.to_string());
                    eprintln!("Retrying...");
                    continue;
                }
                break;
            };
        };
        executor::block_on(executor);
    }));

    loop {
        scheduler.tick();
        std::thread::sleep(Duration::from_millis(500));
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

    let mut rng = rand::thread_rng();
    for i in 0..ITERATIONS {
        let palette = palettes.choose(&mut rng).unwrap();
        println!("generating image {i} using palette [{}]...", palette.0);

        let (image, date) = dither_random_apod_image(&mut rng, &api_key, palette, USE_HD).unwrap();

        match DO_WITH_IMAGE {
            ImageUser::Save => save_image_locally(image, palette.0, &date)?,
            ImageUser::Cohost => dispatch_apod_image_to_cohost(image, &session, palette.0, palette.1.clone(), date).await?,
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

async fn dispatch_apod_image_to_cohost(mut image: DynamicImage, session: &Session, palette_name: &str, palette: Vec<Rgb>, date: String) -> Result<(), Box<dyn Error>> {
    let (mut width, mut height) = image.dimensions();

    if (width * height) > (1920 * 1080) {
        fn resize_image(image: &DynamicImage, factor: f32) -> DynamicImage {
            let (x, y) = image.dimensions();
            let mul = |int: u32, float: f32| (int as f32 * float) as u32;
            image.resize(mul(x, factor), mul(y, factor), imageops::Gaussian)
        }
        
        fn resize_image_with_max_dim(image: &DynamicImage, maxdim: u32) -> DynamicImage {
            let (x, y) = image.dimensions();
            if maxdim < x.max(y) {
                resize_image(&image, maxdim as f32 / x.max(y) as f32)
            } else {
                image.clone()
            }
        }

        image = resize_image_with_max_dim(&image, 1920);
        (width, height) = image.dimensions();
    }

    let temp_file_path = "._temporary.nasa.apod.result.png";
    image.save(temp_file_path)?;

    let metadata = eggbug::MediaMetadata::Image { 
        width: Some(width),
        height: Some(height), 
    };

    // let bytes = image.into_bytes();
    // let mut attachment = Attachment::new(bytes, format!("{date}-{palette_name}.png"), "image/png".into(), metadata);
    let mut attachment = Attachment::new_from_file(temp_file_path, "image/png".into(), Some(metadata)).await?;
    attachment.alt_text = Some(format!("Astronomy Photo Of the Day for: {date}, dithered using {palette_name}."));

    let palette_html = generate_palette_html(palette);

    let mut post = Post {
        adult_content: false,
        headline: format!("{date}"),
        ask: None,
        attachments: vec![
            attachment
        ],
        markdown: format!("**Palette:** *{palette_name}*\n{palette_html}"),
        tags: vec![
            "apod".into(),
            "astronomy photo of the day".into(),
            "nasa".into(),
            "dithering".into(),
            "dither".into(),
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