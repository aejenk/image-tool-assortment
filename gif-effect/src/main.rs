use std::fs::File;
use image::{codecs::gif::{GifDecoder, GifEncoder}, AnimationDecoder};
use image_effects::prelude::{Filter, Affectable, Dither};
use palettes::palettes;

mod frame;
mod palettes;

fn main() {
    let palettes = palettes();

    // let palette = vec![RGB::BLACK, RGB::WHITE, RGB::CYAN, RGB::MAGENTA, RGB::YELLOW];

    let file = File::open("./gif-effect/data/inputs/link.gif").unwrap();
    let decoder = GifDecoder::new(file).unwrap();
    let frames = decoder.into_frames();
    let frames = frames.collect_frames().expect("Error decoding gif");

    for (name, palette) in palettes.iter() {
        println!("working for palette: {name}");
        let frames = frames.clone().into_iter()
            .map(|frame| frame
                .apply(&Filter::Brighten( 0.3))
                .apply(&Filter::Saturate( 0.1))
                .apply(&Filter::Contrast( 1.8))
                // .apply(&Filter::RotateHue(180.0))
                .apply(&Dither::Bayer(8, &palette)))
            .collect::<Vec<_>>();

        let file_out = File::create(format!("./gif-effect/data/output-{name}.gif")).unwrap();
        let mut encoder = GifEncoder::new(file_out);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
        encoder.encode_frames(frames.into_iter()).unwrap();
    }
}   

