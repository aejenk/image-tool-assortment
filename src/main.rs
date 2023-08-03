use image_filters::{
    hsl_gradient_map,
    pixel::{hsl::HslPixel, rgb::colours as RGB},
    prelude::*,
    utils::{
        image::{
            load_image, load_image_from_url_with_max_dim, load_image_with_max_dim, resize_image,
        },
        ImageFilterResult,
    },
};

// this file is essentially for testing / running the code, more than providing an actual reusable binary

fn main() -> ImageFilterResult<()> {
    let gradient = hsl_gradient_map![
        0.00 => sat: 0.0, lum: 0.0, hue: 0.0,
        0.30 => sat: 0.8, lum: 0.3, hue: 280.0,
        0.60 => sat: 0.8, lum: 0.6, hue: 200.0,
        0.80 => sat: 0.8, lum: 0.8, hue: 40.0,
        1.00 => sat: 0.0, lum: 1.0, hue: 260.0
    ];

    let pastel_palette = (
        "pastel",
        [
            RGB::CYAN.mix(0.7, &RGB::BLUE).build_gradient(10),
            RGB::PINK.mix(0.7, &RGB::RED).build_gradient(10),
        ]
        .concat(),
    );

    let rust_palette = (
        "rust",
        [
            RGB::ORANGE.mix(0.2, &RGB::RED).build_gradient(5),
            RGB::ORANGE.mix(0.5, &RGB::RED).build_gradient(5),
            RGB::ORANGE.mix(0.9, &RGB::RED).build_gradient(5),
            vec![RGB::BLACK],
        ]
        .concat(),
    );

    let nb_palette = (
        "nb",
        [
            RGB::GOLD.build_gradient(3),
            RGB::PURPLE.build_gradient(10),
            RGB::GOLD.build_gradient_mix(&RGB::PURPLE, 10),
            vec![RGB::BLACK, RGB::WHITE],
        ]
        .concat(),
    );

    let palettes = [&pastel_palette, &rust_palette, &nb_palette];

    let link_to_image = "https://scied.ucar.edu/sites/default/files/styles/half_width/public/2021-10/cumulus-clouds.jpg.webp?itok=HkQfuWxM";
    let image = load_image_from_url_with_max_dim(link_to_image, 1080)?.apply(Filter::Contrast(1.3));

    for (name, palette) in palettes.into_iter() {
        image
            .clone()
            .apply(Dither::Bayer(8, &palette))
            .save(format!("data/output-{}.png", name))?;
    }

    Ok(())
}
