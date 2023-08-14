use std::error::Error;

use image::{GenericImageView};
use image_effects::{
    prelude::*,
    utils::image::ImageRequest, gradient_map, GradientMap,
};
use palette::{Lch, IntoColor, rgb::Rgb};

mod palettes;

// this file is essentially for testing / running the code, more than providing an actual reusable binary

fn main() -> Result<(), Box<dyn Error>> {
    let palettes = palettes::palettes();

    let image = ImageRequest::File {
        file: "./basic/data/input.png",
        max_dim: Some(1920),
    }.perform()?;

    image.save("./basic/data/__SOURCE.png")?;

    let gradient: GradientMap<Rgb> = gradient_map!(
        0.00 => Lch::new(0.0, 0.0, 0.0).into_color(),
        0.30 => Lch::new(30.0, 80.0, 330.0).into_color(),
        0.60 => Lch::new(60.0, 80.0, 330.0).into_color(),
        0.80 => Lch::new(80.0, 80.0, 200.0).into_color(),
        1.00 => Lch::new(100.0, 0.0, 0.0).into_color()
    );

    for (name, palette) in palettes.iter() {
        println!("Palette: {}", name);
        image
            .clone()
            .apply(&Filter::Brighten(-0.2))
            .apply(&Filter::Contrast( 1.5))
            // .apply(&Filter::GradientMap(&gradient))
            // .apply(&Filter::MultiplyHue(3.0))
            // .apply(&Filter::RotateHue(180.0))
            .apply(&Dither::Bayer(8, palette))
            // .apply(&Dither::Atkinson(palette))
            // .apply(&Filter::Saturate(0.2))
            // .apply(&Filter::Contrast(2.5))
            .apply(&Filter::RotateHue(240.0))
            // .apply(&Filter::MultiplyHue(80.0))
            .save(format!("./basic/data/output-{}.png", name))?;
    }

    // let gradient = generate_hue_gradient(230.0);
    // print_gradient_grid(gradient);

    Ok(())
}