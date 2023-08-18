use std::{fs::File, error::Error};
use common_utils::{palette::palettes, image::GifRequest};
use image::codecs::gif::GifEncoder;
use image_effects::{prelude::*, dither::bayer::Bayer};

fn main() -> Result<(), Box<dyn Error>> {
    let palettes = palettes();

    let frames = GifRequest::Url {
        url: "https://media.tenor.com/ndOR1gN4Q_cAAAAd/uap-dmt.gif",
    }.perform()?;

    for (name, palette) in palettes {
        println!("working for palette: {name}");
        let frames = frames.clone().into_iter()
            .map(|frame| frame
                .apply(&filters::Brighten(-0.25))
                .apply(&filters::Saturate( 0.1))
                .apply(&filters::Contrast( 2.5))
                .apply(&filters::MultiplyHue(20.0))
                // .apply(&Filter::RotateHue(180.0))
                .apply(&Bayer::new(2, palette.clone())))
            .collect::<Vec<_>>();

        let file_out = File::create(format!("./gif-effect/data/output-{name}.gif")).unwrap();
        let mut encoder = GifEncoder::new(file_out);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        encoder.encode_frames(frames.into_iter()).unwrap();
    }

    Ok(())
}   