use std::{fs::File, error::Error};
use common_utils::{palette::generate_random_palette, image::{ImageRequest}, effectlog::{LogEntry, ExecLog}};
use image::{codecs::gif::GifEncoder, Frame};
use image_effects::{prelude::*, dither::bayer::Bayer};
use palette::named;
use rand::{rngs::StdRng, SeedableRng, Rng, seq::SliceRandom};

fn main() -> Result<(), Box<dyn Error>> {

    const TARGET: &str = "https://media.tenor.com/z7CgyBnsPAYAAAAC/sunset-cool.gif";

    let frames = ImageRequest::new(TARGET.into())
        .gif()
        .url()
        .perform()?
        .into_gif()?;

    generate_gifs_with_n_random_palettes(frames, 250)?;
    // generate_gifs_with_alternating_palettes(frames, 250)?;

    // for (name, palette) in palettes {
    //     println!("working for palette: {name}");
    //     let frames = frames.clone().into_iter()
    //         .map(|frame| frame
    //             .apply(&filters::Brighten(-0.25))
    //             .apply(&filters::Saturate( 0.1))
    //             .apply(&filters::Contrast( 2.5))
    //             .apply(&filters::MultiplyHue(20.0))
    //             // .apply(&Filter::RotateHue(180.0))
    //             .apply(&Bayer::new(2, palette.clone())))
    //         .collect::<Vec<_>>();

    //     let file_out = File::create(format!("./gif-effect/data/output-{name}.gif")).unwrap();
    //     let mut encoder = GifEncoder::new(file_out);
    //     encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
    //     encoder.encode_frames(frames.into_iter()).unwrap();
    // }

    Ok(())
}   

fn generate_gifs_with_alternating_palettes(frames: Vec<Frame>, n: usize) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::from_entropy();

    for i in 1..n {
        println!("palette {i} / {n}");

        let mut log = ExecLog::new();

        let mut effects: Vec<Box<dyn Effect<Frame>>> = Vec::new();

        // effects.push(Box::new(filters::Contrast(10.0)));

        if rng.gen_bool(0.1) {
            effects.push(Box::new(filters::Contrast(-1.0)));
        }

        if rng.gen_bool(0.5) {
            let contrast_factor = rng.gen_range(1.0..3.0);    
            log.add_entry(LogEntry::effect("contrast".into(), contrast_factor.to_string()));
            effects.push(Box::new(filters::Contrast(contrast_factor)));
        }
        if rng.gen_bool(0.5) {
            let luma_factor = rng.gen_range(-0.3..0.3);
            log.add_entry(LogEntry::effect("brighten".into(), luma_factor.to_string()));
            effects.push(Box::new(filters::Brighten(luma_factor)));
        }
        if rng.gen_bool(0.5) {
            let hue_shift = rng.gen_range(0.0..360.0); 
            log.add_entry(LogEntry::effect("hue-rotate".into(), hue_shift.to_string()));
            effects.push(Box::new(filters::HueRotate(hue_shift)));
        }
        if rng.gen_bool(0.5) {
            let chroma_factor = rng.gen_range(-0.3..0.3); 
            log.add_entry(LogEntry::effect("saturate".into(), chroma_factor.to_string()));
            effects.push(Box::new(filters::Saturate(chroma_factor)));
        }

        let possible_matrix_sizes = [2, 4, 8, 16];
        let matrix_size = possible_matrix_sizes.choose(&mut rng).unwrap();
        log.add_entry(LogEntry::effect("dither(bayer)".into(), format!("matrix-size({matrix_size})")));
        log.add_entry(LogEntry::effect(format!("{} generated palettes.", frames.len()), "[omitted]".to_string()));

        let frames = frames.clone().into_iter().map(|mut frame| {
            let (palette, _) = generate_random_palette(&mut rng);

            let palette = [palette, vec![named::BLACK.into_format(), named::WHITE.into_format()]].concat();
            effects.push(Box::new(Bayer::new(*matrix_size as usize, palette)));

            for effect in &effects {
                frame = effect.affect(frame);
            }

            effects.pop();
            frame
        }).collect::<Vec<_>>();

        let file_out = File::create(format!("./gif-effect/data/gen-output-{i}.gif")).unwrap();
        let mut encoder = GifEncoder::new(file_out);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        encoder.encode_frames(frames.into_iter()).unwrap();

        // log.write_to(format!("./gif-effect/data/gen-output-{i}.log.txt").as_str())?;
    }

    Ok(())
}

fn generate_gifs_with_n_random_palettes(frames: Vec<Frame>, n: usize) -> Result<(), Box<dyn Error>> {
    let mut rng = StdRng::from_entropy();

    for i in 0..n {
        println!("palette {i} / {n}");

        let (palette, mut log) = generate_random_palette(&mut rng);

        let mut effects: Vec<Box<dyn Effect<Frame>>> = Vec::new();

        if rng.gen_bool(0.1) {
            effects.push(Box::new(filters::Contrast(-1.0)));
        }

        if rng.gen_bool(0.5) {
            let contrast_factor = rng.gen_range(1.0..3.0);    
            log.add_entry(LogEntry::effect("contrast".into(), contrast_factor.to_string()));
            effects.push(Box::new(filters::Contrast(contrast_factor)));
        }
        if rng.gen_bool(0.5) {
            let luma_factor = rng.gen_range(-0.3..0.3);
            log.add_entry(LogEntry::effect("brighten".into(), luma_factor.to_string()));
            effects.push(Box::new(filters::Brighten(luma_factor)));
        }
        if rng.gen_bool(0.5) {
            let hue_shift = rng.gen_range(0.0..360.0); 
            log.add_entry(LogEntry::effect("hue-rotate".into(), hue_shift.to_string()));
            effects.push(Box::new(filters::HueRotate(hue_shift)));
        }
        if rng.gen_bool(0.5) {
            let chroma_factor = rng.gen_range(-0.3..0.3); 
            log.add_entry(LogEntry::effect("saturate".into(), chroma_factor.to_string()));
            effects.push(Box::new(filters::Saturate(chroma_factor)));
        }

        let possible_matrix_sizes = [2, 4, 8, 16];
        let matrix_size = possible_matrix_sizes.choose(&mut rng).unwrap();
        effects.push(Box::new(Bayer::new(*matrix_size as usize, palette)));
        log.add_entry(LogEntry::effect("dither(bayer)".into(), format!("matrix-size({matrix_size})")));

        let frames = frames.clone().into_iter().enumerate().map(|(i, mut frame)| {
            frame = filters::HueRotate((i as f32 / frames.len() as f32) * 360.0).affect(frame);
            for effect in &effects {
                frame = effect.affect(frame);
            }
            frame
        }).collect::<Vec<_>>();

        let file_out = File::create(format!("./gif-effect/data/gen-output-{i}.gif")).unwrap();
        let mut encoder = GifEncoder::new(file_out);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        encoder.encode_frames(frames.into_iter()).unwrap();

        // log.write_to(format!("./gif-effect/data/gen-output-{i}.log.txt").as_str())?;
    }

    Ok(())
}