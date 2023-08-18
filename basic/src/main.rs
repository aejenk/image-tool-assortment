use std::error::Error;

use common_utils::image::ImageRequest;
use image_effects::{
    prelude::*,
    gradient_map, GradientMap, dither::bayer::Bayer,
};
use palette::{Lch, IntoColor, rgb::Rgb};

// this file is essentially for testing / running the code, more than providing an actual reusable binary

fn main() -> Result<(), Box<dyn Error>> {
    let palettes = common_utils::palette::palettes();

    // let image = ImageRequest::File {
    //     file: "./basic/data/input.jpg",
    //     max_dim: Some(720),
    // }.perform()?;

    let image = ImageRequest::Url { 
        url: "https://media.discordapp.net/attachments/766775857372463144/1141927756586831882/20230817_225216.jpg",
        max_dim: Some(720), 
    }.perform()?;

    image.save("./basic/data/__SOURCE.png")?;

    let gradient: GradientMap<Rgb> = gradient_map!(
        0.00 => Lch::new(0.0, 0.0, 0.0).into_color(),
        0.30 => Lch::new(30.0, 80.0, 330.0).into_color(),
        0.60 => Lch::new(60.0, 80.0, 330.0).into_color(),
        0.80 => Lch::new(80.0, 80.0, 200.0).into_color(),
        1.00 => Lch::new(100.0, 0.0, 0.0).into_color()
    );

    for (name, palette) in palettes {
        println!("Palette: {}", name);
        image
            .clone()
            // .apply(&Filter::Brighten( 0.1))
            // .apply(&Filter::Contrast( 2.0))
            // .apply(&Filter::GradientMap(&gradient))
            // .apply(&Filter::MultiplyHue(3.0))
            // .apply(&Filter::RotateHue(60.0))
            .apply(&Bayer::new(8, palette))
            // .apply(&Dither::Atkinson(palette))
            // .apply(&Filter::Saturate(0.2))
            // .apply(&Filter::Contrast(2.5))
            // .apply(&Filter::RotateHue(240.0))
            // .apply(&Filter::MultiplyHue(80.0))
            .save(format!("./basic/data/output-{}.png", name))?;
    }

    // let gradient = generate_hue_gradient(230.0);
    // print_gradient_grid(gradient);

    Ok(())
}