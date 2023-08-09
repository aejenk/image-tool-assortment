use image::{Frame, DynamicImage};
use image_effects::prelude::{EffectInput, Filter, Dither};

pub struct GifFrame(pub Frame);

impl<'a> EffectInput<Filter<'a>> for GifFrame {
    fn run_through(&self, effect: &Filter<'a>) -> Self {
        let left = self.0.left();
        let top = self.0.top();
        let delay = self.0.delay();
        let rgba = DynamicImage::ImageRgba8(self.0.buffer().clone()).run_through(effect).into_rgba8();
        GifFrame(Frame::from_parts(rgba, left, top, delay))
    }
}

impl<'a> EffectInput<Dither<'a>> for GifFrame {
    fn run_through(&self, effect: &Dither<'a>) -> Self {
        let left = self.0.left();
        let top = self.0.top();
        let delay = self.0.delay();
        let rgba = DynamicImage::ImageRgba8(self.0.buffer().clone()).run_through(effect).into_rgba8();
        GifFrame(Frame::from_parts(rgba, left, top, delay))
    }
}