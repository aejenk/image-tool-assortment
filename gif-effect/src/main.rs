use std::fs::File;

use frame::GifFrame;
use gif::Encoder;
use image_effects::prelude::{Filter, Affectable};

mod frame;

fn main() {
    let mut decoder = gif::DecodeOptions::new();

    decoder.set_color_output(gif::ColorOutput::RGBA);

    let file = File::open("./gif-effect/data/rabbit-magnet.gif").unwrap();

    let mut decoder = decoder.read_info(file).unwrap();

    let gif_width = decoder.width();
    let gif_height = decoder.height();

    let mut output = File::create("./gif-effect/data/rabbit-contrast.gif").unwrap();
    let mut encoder = Encoder::new(&mut output, gif_width, gif_height, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    let mut _frame_count = 0;
    while let Some(frame) = decoder.read_next_frame().unwrap() {
        if frame.width != gif_width || frame.height != gif_height {
            continue;
        }

        let frame = GifFrame(frame.clone())
            .apply(&Filter::Contrast(1.5))
            .apply(&Filter::Saturate(1.0));

        encoder.write_frame(&frame.0).unwrap();
    }
}   
