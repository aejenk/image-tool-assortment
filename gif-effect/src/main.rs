use std::{fs::File, error::Error};
use image::codecs::gif::GifEncoder;
use image_effects::prelude::{Filter, Affectable, Dither};
use image_effects::utils::image::GifRequest;
use palettes::palettes;

mod frame;
mod palettes;

fn main() -> Result<(), Box<dyn Error>> {
    let palettes = palettes();

    let frames = GifRequest::Url {
        url: "https://media.tenor.com/DxdIH1z9iw4AAAAC/cat-kill.gif",
    }.perform()?;

    for (name, palette) in palettes.iter() {
        println!("working for palette: {name}");
        let frames = frames.clone().into_iter()
            .map(|frame| frame
                .apply(&Filter::Brighten( 0.2))
                .apply(&Filter::Saturate( 0.2))
                .apply(&Filter::Contrast(-4.0))
                // .apply(&Filter::MultiplyHue(20.0))
                // .apply(&Filter::RotateHue(180.0))
                .apply(&Dither::Bayer(2, &palette)))
            .collect::<Vec<_>>();

        let file_out = File::create(format!("./gif-effect/data/output-{name}.gif")).unwrap();
        let mut encoder = GifEncoder::new(file_out);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        encoder.encode_frames(frames.into_iter()).unwrap();
    }

    Ok(())
}   