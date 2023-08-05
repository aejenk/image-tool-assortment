use image_filters::{
    pixel::{rgb::{colours as RGB, RgbPixel}, oklch::OklchPixel},
    prelude::*,
    utils::{
        image::load_image_from_url_with_max_dim,
        ImageFilterResult,
    },
};

// this file is essentially for testing / running the code, more than providing an actual reusable binary

fn main() -> ImageFilterResult<()> {
    // let build_gradient = RgbPixel::build_gradient_using_hsl;
    let build_gradient = RgbPixel::build_gradient_using_oklch;

    let mut palettes = vec![
        (
            "pastel",
            [
                build_gradient(&RGB::CYAN.mix(0.8, &RGB::BLUE), 10),
                build_gradient(&RGB::PINK.mix(0.8, &RGB::RED), 10),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "carrot",
            [
                build_gradient(&RGB::ORANGE, 10),
                build_gradient(&RGB::GREEN, 10),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "nb",
            [
                build_gradient(&RGB::GOLD, 10),
                build_gradient(&RGB::PURPLE, 10),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "sunsky",
            [
                build_gradient(&RGB::ORANGE, 10),
                build_gradient(&RGB::BLUE.mix(0.2, &RGB::CYAN), 10),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
        (
            "cosmos",
            [
                build_gradient(&RGB::BLUE, 10),
                build_gradient(&RGB::PURPLE, 10),
                build_gradient(&RGB::ROSE, 10),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat(),
        ),
    ];

    palettes.push((
        "all",
        palettes.iter().map(|col| (&col.1).clone()).collect::<Vec<_>>().concat(),
    ));

    let link_to_image = "https://www.bibalex.org/SCIplanet/Attachments/images/big-orange-sun-4.jpg";
    let image = load_image_from_url_with_max_dim(link_to_image, 1080)?.apply(Filter::Contrast(1.3));

    image.save("data/_original.png")?;

    for (name, palette) in palettes.into_iter() {
        image
            .clone()
            // .apply(Filter::Contrast( 1.2))
            .apply(Dither::Bayer(8, &palette))
            .save(format!("data/output-{}.png", name))?;
    }

    Ok(())
}
