use image_filters::{
    prelude::*,
    prelude::SrgbColour as RGB,
    utils::{
        image::load_image_from_url_with_max_dim,
        ImageFilterResult,
    }, colour::utils::GradientMethod, hsl_gradient_map, GradientMap
};

pub mod nasa;

use palette::{Lch, Srgb, FromColor};

// this file is essentially for testing / running the code, more than providing an actual reusable binary

fn main() -> ImageFilterResult<()> {
    const GRADIENT_METHOD: GradientMethod = GradientMethod::LCH;

    let mut palettes = vec![
        (
            "pastel",
            [
                RGB::CYAN.build_gradient(10, GRADIENT_METHOD),
                RGB::PINK.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "carrot",
            [
                RGB::ORANGE.build_gradient(10, GRADIENT_METHOD),
                RGB::GREEN.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "nb",
            [
                RGB::GOLD.build_gradient(10, GRADIENT_METHOD),
                RGB::PURPLE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "sunsky",
            [
                RGB::ORANGE.build_gradient(10, GRADIENT_METHOD),
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "cosmos",
            [
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                RGB::PURPLE.build_gradient(10, GRADIENT_METHOD),
                RGB::ROSE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
    ];

    palettes.push((
        "all",
        palettes.iter().map(|col| (&col.1).clone()).collect::<Vec<_>>().concat(),
    ));

    let link_to_image = "https://ugc.berkeley.edu/wp-content/uploads/2016/01/thunderstorm-3625405_1920.jpg";
    let image = load_image_from_url_with_max_dim(link_to_image, 1080)?;

    image.save("data/_original.png")?;

    // let gradient_map: Vec<(Srgb, f32)> = [
    //     (Lch::new(0.0, 100.0, 0.0), 0.00),
    //     (Lch::new(20.0, 100.0, 60.0), 0.20),
    //     (Lch::new(40.0, 100.0, 120.0), 0.40),
    //     (Lch::new(60.0, 100.0, 180.0), 0.60),
    //     (Lch::new(80.0, 100.0, 240.0), 0.80),
    //     (Lch::new(100.0, 100.0, 300.0), 1.00),
    // ]
    //     .iter()
    //     .map(|(colour, th)| (Srgb::from_color(*colour), *th))
    //     .collect::<Vec<_>>();

    for (name, palette) in palettes.into_iter() {
        image
            .clone()
            // .apply(Filter::Saturate( 0.2))
            // .apply(Filter::Contrast( 1.8))
            .apply(Dither::Bayer(8, &palette))
            // .apply(Filter::QuantizeHue(&[
            //     180.0, 200.0, 220.0, 240.0, 300.0, 330.0, 360.0, 30.0
            // ]))
            // .apply(Filter::GradientMap(
            //     &gradient_map
            // ))
            .save(format!("data/output-{}.png", name))?;
    }

    Ok(())
}
