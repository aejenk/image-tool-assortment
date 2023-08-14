use image_effects::{prelude::{SrgbColour as RGB, IntoGradient}, colour::utils::GradientMethod};
use palette::rgb::Rgb;

pub fn palettes<'a>() -> Vec<(&'a str, Vec<Rgb>)> { 
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
            "crisp-nightlife",
            [
                vec![RGB::CYAN, RGB::PINK, RGB::YELLOW, RGB::GOLD, RGB::BLUE, RGB::PURPLE, RGB::WHITE, RGB::BLACK],
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
            "nblofi",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::GOLD, RGB::YELLOW, RGB::PURPLE,],
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
                {
                    let mut red = RGB::RED.clone();
                    red.red = 0.2;
                    red 
                }.build_gradient(40, GRADIENT_METHOD),
                // vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "minty",
            [
                RGB::GREEN.build_gradient(40, GRADIENT_METHOD),
                RGB::GOLD.build_gradient(4, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        ),
        (
            "corru",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::CYAN, RGB::MAGENTA, RGB::YELLOW],
            ].concat()
        ),
        (
            "zx",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::CYAN, RGB::MAGENTA],
            ].concat()
        ),
        (
            "blue",
            [
                vec![RGB::BLACK, RGB::BLUE],
            ].concat()
        ),
        (
            "red",
            [
                vec![RGB::BLACK, RGB::RED],
            ].concat()
        ),
        (
            "mono",
            [
                vec![RGB::BLACK, RGB::WHITE],
            ].concat()
        ),
        (
            "orangurple",
            [
                vec![RGB::BLACK, RGB::WHITE, RGB::PURPLE, RGB::ORANGE],
            ].concat()
        ),
        (
            "calmfire",
            [
                vec![RGB::WHITE, RGB::ROSE, RGB::ORANGE, RGB::BLACK],
            ].concat()
        ),
        (
            "rcgmby",
            [
                vec![RGB::RED, RGB::CYAN, RGB::GREEN, RGB::MAGENTA, RGB::BLUE, RGB::YELLOW],
            ].concat()
        ),
        (
            "eight-bit",
            [
                vec![RGB::RED, RGB::BLUE, RGB::GREEN, RGB::WHITE, RGB::BLACK],
            ].concat(),
        ),
        (
            "deep-crushed-ocean",
            [
                vec![RGB::BLACK, RGB::PURPLE, RGB::BLUE, RGB::CYAN, RGB::WHITE],
            ].concat(),
        ),
        (
            "falling-bitsun",
            [
                vec![RGB::BLACK, RGB::PURPLE, RGB::RED, RGB::ORANGE, RGB::GOLD, RGB::WHITE],
            ].concat(),
        ),
        (
            "pixeleaf",
            [
                RGB::GREEN.build_gradient(4, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ].concat(),
        )
    ];
    palettes.push((
        "all",
        palettes.iter().map(|col| (&col.1).clone()).collect::<Vec<_>>().concat(),
    ));
    palettes
}

pub fn generate_palette_html(gradient: Vec<Rgb>) -> String {
    let palette_html = gradient.iter().map(|colour| {
        let (r, g, b) = colour.into_format::<u8>().into_components();
        format!("<div style=\"height: 100%; background: rgb({r},{g},{b}); flex-grow: 1; padding:3px;\"></div>")
    }).collect::<Vec<_>>().concat();

    vec![
        "<div style:\"width: 100%; display: flex; flex-wrap: wrap;\">",
        &palette_html,
        "</div>",
    ].concat()
}