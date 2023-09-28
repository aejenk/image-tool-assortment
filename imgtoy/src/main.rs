use std::{error::Error, fs::File};

use effects::EffectKind;
use image::{DynamicImage, Frame, codecs::gif::GifEncoder};
use image_effects::prelude::Effect;
use rand::{rngs::StdRng, SeedableRng};
use serde::Deserialize;
use source::{SourceKind, MediaType, Source};
use indicatif::{ProgressBar, ProgressStyle};

use crate::effects::parse_effects;

mod effects;
mod source;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    if args.len() != 2 {
        panic!("Expected a single arg which represents the filepath of the configuration file.");
    }

    let config = std::fs::read_to_string(args.nth(1).unwrap())?;

    let yaml: serde_yaml::Value = serde_yaml::from_str(&config)?;

    let mut rng = StdRng::from_entropy();

    let source = parse_source(&yaml);

    let output = yaml
        .get("output").expect("[output] is required.")
        .as_mapping().expect("[output] must be a mapping / object.");

    let out_path = output.get("path").expect("[output.path] must be specified.")
        .as_str().expect("[output.path] must be a string.");

    let iterations = output.get("n").expect("[n] must be specified.")
        .as_u64().expect("[output.n] must be a positive integer.");

    let media = source.perform()?;

    let bar = ProgressBar::new(iterations);
    bar.set_style(ProgressStyle::with_template("[{eta:>8} remaining...] {pos:>4}/{len:4} {bar:40.cyan/blue} {msg}")
        .unwrap());

    for i in 0..iterations {
        bar.inc(1);
        match source.media_type {
            MediaType::Image => {
                let effects = parse_effects::<DynamicImage>(&mut rng, &yaml);
                let mut image = (&media).clone().into_image().unwrap();
                for effect in &effects {
                    bar.tick();
                    image = effect.affect(image);
                }
                image.save(format!("{out_path}-{i:<05}.png"))?;
            },
            MediaType::Gif => {
                let effects = parse_effects::<Frame>(&mut rng, &yaml);
                let frames = (&media).clone().into_gif().unwrap();
                let frames_amnt = frames.len();
                let frames = frames.into_iter().enumerate().map(|(i, mut frame)| {
                    bar.set_message(format!("frame {i} of {frames_amnt}"));
                    for effect in &effects {
                        bar.tick();
                        frame = effect.affect(frame);
                    }
                    frame
                }).collect::<Vec<_>>();

                let file_out = File::create(format!("{out_path}-{i:<05}.gif")).unwrap();
                let mut encoder = GifEncoder::new(file_out);
                encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
                encoder.encode_frames(frames.into_iter()).unwrap();
            }
        }
    }

    let dur = bar.duration();
    let h = dur.as_secs() / (60 * 60);
    let m = dur.as_secs() / (60) % 60;
    let s = dur.as_secs() % 60;
    println!("done in {h:0>2}:{m:0>2}:{s:0>2}!");

    Ok(())
}

fn parse_source(root_value: &serde_yaml::Value) -> Source {
    let source = root_value
        .get("source").expect("[source] was not present - is required.")
        .as_mapping().expect("[source] must be a map - wasn't.");

    let url = source.get("url");
    let file = source.get("file");

    if url.is_some() && file.is_some() {
        panic!("only one of [source.url] and [source.file] can be present.");
    } else if url.is_none() && file.is_none() {
        panic!("at least one of [source.url] or [source.file] must be present");
    }

    let url = url.map(|target| target.as_str().expect("[source.url] must be a string - wasn't."));
    let file = file.map(|target| target.as_str().expect("[source.file] must be a string - wasn't."));

    let media_type = source.get("media_type").expect("[source.media_type] was not present - is required.")
        .as_str().expect("[source.media_type] must be a string - wasn't.");

    let media_type = match media_type {
        "image" => MediaType::Image,
        "gif" => MediaType::Gif,
        _ => panic!("[source.media_type] must be 'image' or 'gif' - was actually {media_type}"),
    };

    let source_kind = if let Some(url) = url { SourceKind::Url(url.into()) }
    else if let Some(file) = file { SourceKind::File(file.into()) }
    else { unreachable!() };

    let max_dim = if let Some(max_dim) = source.get("max_dim") {
        if let Some(max_dim) = max_dim.as_u64() {
            Some(max_dim as usize)
        } else {
            panic!("[max_dim] must be a positive integer.");
        }
    } else { None };

    Source { source: source_kind, media_type, max_dim }
}
