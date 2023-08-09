use gif::Frame;
use image_effects::prelude::{EffectInput, Filter};

pub struct GifFrame<'a>(pub Frame<'a>);

impl<'a> EffectInput<Filter<'a>> for GifFrame<'a> {
    fn run_through(&self, effect: &Filter<'a>) -> Self {

        let mut new_pixels: Vec<u8> = vec![];
        let old_frame = &self.0;

        println!("OLD: {}", old_frame.buffer.len());

        for i in 0..(old_frame.buffer.len() / 4) {
            let mut rgb = [0_u8; 3];
            let real_index = i * 4;

            for j in 0..3 {
                rgb[j] = old_frame.buffer[real_index + j];
            }

            // process effect here
            // the major problem being that the dithering algorithms only work for DynamicImages currently.
            // and even then, we'd need to encode _by frame_ which can be a bit annoying to work out.
            // in essence, we need to generalize each ditherer to accept a 2D matrix of [u8; 3].
            let rgb = rgb.run_through(effect);

            new_pixels.push(rgb[0]);
            new_pixels.push(rgb[1]);
            new_pixels.push(rgb[2]);
            new_pixels.push(old_frame.buffer[real_index + 3]);
        }

        GifFrame(Frame::from_rgba_speed(old_frame.width, old_frame.height, &mut new_pixels, 20))
    }
}