use image_filters::{
    prelude::*,
    prelude::SrgbColour as RGB,
    utils::{
        image::{load_image_from_url_with_max_dim, load_image_with_max_dim},
        ImageFilterResult,
    }, colour::utils::GradientMethod
};

use palette::{Lch, Srgb, FromColor};

// this file is essentially for testing / running the code, more than providing an actual reusable binary

fn example() -> ImageFilterResult<()> {
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
            "nightlife",
            [
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                RGB::CYAN.build_gradient(10, GRADIENT_METHOD),
                RGB::PINK.build_gradient(10, GRADIENT_METHOD),
                RGB::ROSE.build_gradient(10, GRADIENT_METHOD),
                RGB::YELLOW.build_gradient(10, GRADIENT_METHOD),
                RGB::GOLD.build_gradient(10, GRADIENT_METHOD),
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
                RGB::PURPLE.build_gradient(30, GRADIENT_METHOD),
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
            "depth",
            [
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                RGB::PURPLE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "refresh",
            [
                RGB::BLUE.build_gradient(10, GRADIENT_METHOD),
                RGB::CYAN.build_gradient(10, GRADIENT_METHOD),
                RGB::AQUAMARINE.build_gradient(10, GRADIENT_METHOD),
                RGB::GREEN.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "nebula",
            [
                RGB::RED.build_gradient(10, GRADIENT_METHOD),
                RGB::ROSE.build_gradient(10, GRADIENT_METHOD),
                RGB::PURPLE.build_gradient(10, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "dragon",
            [
                RGB::RED.build_gradient(40, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        )
    ];

    palettes.push((
        "all",
        palettes.iter().map(|col| (&col.1).clone()).collect::<Vec<_>>().concat(),
    ));

    const IMAGE_LINK: &'static str = "./data/daigo.png";
    const IS_URL: bool = false;
    const MAX_DIM: u32 = 1080;

    let image = if IS_URL {
        load_image_from_url_with_max_dim(IMAGE_LINK, MAX_DIM)?
    } else {
        load_image_with_max_dim(IMAGE_LINK, MAX_DIM)?
    };

    image.save("data/_original.png")?;

    let gradient_map: Vec<(Srgb, f32)> = [
        (Lch::new(0.0, 100.0, 0.0), 0.00),
        (Lch::new(60.0, 100.0, 0.0), 0.20),
        (Lch::new(60.0, 100.0, 90.0), 0.40),
        (Lch::new(60.0, 100.0, 180.0), 0.60),
        (Lch::new(80.0, 100.0, 270.0), 0.80),
        (Lch::new(100.0, 100.0, 360.0), 1.00),
    ]
        .iter()
        .map(|(colour, th)| (Srgb::from_color(*colour), *th))
        .collect::<Vec<_>>();

    for (name, palette) in palettes.into_iter() {
        image
            .clone()
            // .apply(Filter::Brighten( 0.3))
            .apply(Filter::Contrast( 1.2))
            // .apply(Filter::GradientMap(
            //     &gradient_map
            // ))
            .apply(Dither::Bayer(8, &palette))
            // .apply(Filter::RotateHue(100.0))
            // .apply(Filter::QuantizeHue(&[
            //     180.0, 200.0, 220.0, 240.0, 300.0, 330.0, 360.0, 30.0
            // ]))
            .save(format!("data/output-{}.png", name))?;
    }

    Ok(())
}
