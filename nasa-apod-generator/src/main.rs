use std::{error::Error, time::Duration};

use clokwerk::{AsyncScheduler, TimeUnits};
use common_utils::{palette::{palettes, generate_palette_html, generate_random_palette}, image::resize_image_with_max_dim};
use image::GenericImageView;
use image_effects::{prelude::*, dither::bayer::Bayer};
use nasa::ApodResponse;
use palette::rgb::Rgb;
use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng, Rng};
use dotenv::dotenv;
use eggbug::{Session, Post, Attachment};
use chrono;
use async_std::task;


pub mod nasa;
mod utils;

use crate::nasa::get_random_apod;

const ITERATIONS: usize = 1;

static mut RUNNABLE: bool = true;
const SAFEGUARD_SECS: u64 = 60 * 60; // hour

#[tokio::main]
async fn main() {
    let mut scheduler = AsyncScheduler::new();

    println!("----- Launched on: {:?}", chrono::offset::Local::now());

    // for testing
    // let _ = execute().await;

    scheduler.every(1.day()).run(|| async {
        if unsafe { !RUNNABLE } {
            println!("----- Already executed recently. Skipping iteration...");
            return;
        }
        println!("----- Executing on: {:?}", chrono::offset::Local::now());

        let mut tries = 0;

        loop {
            if let Err(error) = execute().await {
                tries = tries + 1;
                eprintln!("Try: {tries}... Error encountered: {}", error.to_string());

                if tries > 3 {
                    eprintln!("Maxxed out tries. Skipping iteration...");
                    return;
                }
                
                eprintln!("Retrying...");
                continue;
            }
            println!("Completed loop! Returning to scheduler...\n");
            
            unsafe { RUNNABLE = false; }
            
            task::spawn(async move {
                task::sleep(Duration::from_secs(SAFEGUARD_SECS)).await;
                unsafe { RUNNABLE = true; }
            });

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
    let api_key = std::env::var("NASA_API_KEY").expect("NASA_API_KEY must be set in the environment/.env.");
    let email = std::env::var("COHOST_EMAIL").expect("COHOST_EMAIL must be set in the environment/.env.");
    let password = std::env::var("COHOST_PASSWORD").expect("COHOST_PASSWORD must be set in the environment/.env.");

    let session = Session::login(&email, &password).await?;

    let mut rng = StdRng::from_entropy();

    let mut _palettes = Vec::new();

    if rng.gen_bool(0.25) {
        println!("generating palettes...");
        for i in 0..50 {
            _palettes.push((
                format!("generated({i})"),
                generate_random_palette(&mut rng).0
            ));
        }
    } else {
        println!("using predetermined palettes...");
        _palettes = palettes().into_iter().map(|(n, p)| (n.into(), p)).collect();
    }

    let palettes = _palettes;

    for i in 0..ITERATIONS {
        let palette = palettes.choose(&mut rng).unwrap();
        println!("generating image {i} using palette [{}]...", palette.0);

        let mut response = get_random_apod(&mut rng, &api_key)?;

        response.image = resize_image_with_max_dim(&response.image, 1080);

        response.image = response.image
            .apply(&Bayer::new(8, palette.1.clone()));

        let image_filename = format!("./nasa-apod-generator/data/nasa-output-{}-{}.png", palette.0, response.date);
        response.image.save_with_format(&image_filename, image::ImageFormat::Png)?;

        dispatch_apod_image_to_cohost(
            response,
            &session,
            &image_filename,
            (palette.0.as_str(), palette.1.clone()),
        ).await?;
    }

    Ok(())
}

async fn dispatch_apod_image_to_cohost(
    mut response: ApodResponse,
    session: &Session,
    filename: &str,
    (palette_name, palette_cols): (&str, Vec<Rgb>)
) -> Result<(), Box<dyn Error>> 
{
    let (mut width, mut height) = response.image.dimensions();

    if (width * height) > (1920 * 1080) {
        response.image = common_utils::image::resize_image_with_max_dim(&response.image, 720);
        (width, height) = response.image.dimensions();
    }

    let metadata = eggbug::MediaMetadata::Image { 
        width: Some(width),
        height: Some(height), 
    };

    let mut attachment = Attachment::new_from_file(
        filename,
        "image/png".into(),
        Some(metadata)
    ).await?;

    let alt_text = format!("Astronomy Photo Of the Day for: {}, dithered using {palette_name}. Titled: {}", response.date, response.title);
    attachment.alt_text = Some(alt_text.clone());

    let palette_html = generate_palette_html(palette_cols);

    let mut post = Post {
        adult_content: false,
        headline: format!("{} - {}", response.date, response.title),
        ask: None,
        attachments: vec![
            attachment
        ],
        markdown: vec![
            format!("**Explanation:** {}", response.explanation),
            format!("<hr/>\n**Palette:** *{palette_name}*\n{palette_html}"),
            format!("<hr/>\n\n## Original\n\n![{}]({})", response.title, response.url),
        ].join(""),
        tags: vec![
            "apod".into(),
            "astronomy photo of the day".into(),
            "nasa".into(),
            "dithering".into(),
            "dither".into(),
            // "astronomy".into(),
            "bot".into(),
            format!("apod-date({})", response.date),
            format!("palette({})", palette_name),
        ],
        content_warnings: vec![],
        draft: false,
        metadata: None,
    };

    let post_id = session.create_post("ditherpod", &mut post).await?;
    println!("Published post (id: {})", post_id.0);

    Ok(())
}