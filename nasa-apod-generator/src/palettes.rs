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
        ),
        (
            "minty",
            [
                RGB::GREEN.build_gradient(40, GRADIENT_METHOD),
                RGB::GOLD.build_gradient(4, GRADIENT_METHOD),
                vec![RGB::BLACK, RGB::WHITE],
            ]
            .concat()
        )
    ];
    palettes.push((
        "all",
        palettes.iter().map(|col| (&col.1).clone()).collect::<Vec<_>>().concat(),
    ));
    palettes
}