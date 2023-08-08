use image_filters::{prelude::{SrgbColour as RGB, IntoGradient}, colour::utils::GradientMethod};
use palette::{rgb::Rgb, Lch};

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
    ];
    palettes.push((
        "all",
        palettes.iter().map(|col| (&col.1).clone()).collect::<Vec<_>>().concat(),
    ));
    palettes
}

pub fn generate_hue_gradient(hue: f32) -> Vec<Vec<Lch>> {
    (0..8).into_iter().map(|chroma| Lch::new(0.0, 128.0 - (chroma*16) as f32, hue).build_gradient_lch(10)).collect()
}

pub fn print_gradient_grid(gradient: Vec<Vec<Lch>>) {
    let div = gradient.iter().map(|chroma_variant| {
        let html_lines = chroma_variant.iter().map(|color| {
            let (l, c, h) = color.into_components();
            let h = h.into_degrees();
            format!("<div style=\"height: 100%; background: lch({l}% {c} {h}); flex-grow: 1; padding:3px;\"></div>")
        }).collect::<Vec<_>>().join("\n");

        vec![
            "<div style=\"width: 100%; height:50px; display: flex;\">",
            html_lines.as_str(),
            "</div>",
        ].join("\n")
    }).collect::<Vec<_>>().join("\n");

    println!("{div}");
}