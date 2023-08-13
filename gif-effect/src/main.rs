use std::{fs::File, error::Error, io::Read};
use image::{codecs::gif::{GifDecoder, GifEncoder}, AnimationDecoder, Frame};
use image_effects::prelude::{Filter, Affectable, Dither};
use palettes::palettes;

mod frame;
mod palettes;

fn main() -> Result<(), Box<dyn Error>> {
    let palettes = palettes();

    // let palette = vec![RGB::BLACK, RGB::WHITE, RGB::CYAN, RGB::MAGENTA, RGB::YELLOW];

    // let frames = load_gif_from_file("./gif-effect/data/inputs/pinkerton.gif")?;
    let frames = load_gif_from_url("https://media.tenor.com/anFDxPzJV3gAAAAM/cat-running.gif")?;

    for (name, palette) in palettes.iter() {
        println!("working for palette: {name}");
        let frames = frames.clone().into_iter()
            .map(|frame| frame
                // .apply(&Filter::Brighten(-0.1))
                .apply(&Filter::Saturate( 0.3))
                .apply(&Filter::Contrast( 2.5))
                .apply(&Filter::MultiplyHue(3.0))
                // .apply(&Filter::RotateHue(180.0))
                .apply(&Dither::Bayer(8, &palette)))
            .collect::<Vec<_>>();

        let file_out = File::create(format!("./gif-effect/data/output-{name}.gif")).unwrap();
        let mut encoder = GifEncoder::new(file_out);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        encoder.encode_frames(frames.into_iter()).unwrap();
    }

    Ok(())
}   

fn load_gif_from_file(path: &str) -> Result<Vec<Frame>, Box<dyn Error>> {
    let file = File::open(path).unwrap();
    Ok(GifDecoder::new(file)?.into_frames().collect_frames()?)
}

fn load_gif_from_url(url: &str) -> Result<Vec<Frame>, Box<dyn Error>> {
    let mut gif_bytes = reqwest::blocking::get(url)?;
    
    let mut data = Vec::new();
    gif_bytes.read_to_end(&mut data)?;
    
    Ok(GifDecoder::new(data.as_slice())?.into_frames().collect_frames()?)
}