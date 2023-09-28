use std::collections::HashMap;

use image_effects::{prelude::{Effect, filters::{Brighten, HueRotate, Contrast, Saturate, GradientMap, QuantizeHue, MultiplyHue}, IntoGradientLch}, dither::{bayer::Bayer, error::{ErrorPropagator, WithPalette}, ATKINSON, FLOYD_STEINBERG, JARVIS_JUDICE_NINKE, BURKES, STUCKI, SIERRA, SIERRA_TWO_ROW, SIERRA_LITE}};
use palette::{Srgb, Lch, IntoColor, named};
use rand::{Rng, seq::SliceRandom};
use serde_yaml::{Mapping, Sequence};

#[derive(Debug)]
pub enum EffectKind {
    HueRotate,
    Contrast,
    Brighten,
    Saturate,
    GradientMap,
    QuantizeHue,
    MultiplyHue,

    Bayer,
    ErrorPropagator,
}

impl From<&str> for EffectKind {
    fn from(value: &str) -> Self {
        match value {
            "hue-rotate" => EffectKind::HueRotate,
            "contrast" => EffectKind::Contrast,
            "brighten" => EffectKind::Brighten,
            "bayer" => EffectKind::Bayer,
            "saturate" => EffectKind::Saturate,
            "gradient-map" => EffectKind::GradientMap,
            "quantize-hue" => EffectKind::QuantizeHue,
            "multiply-hue" => EffectKind::MultiplyHue,
            _ => EffectKind::ErrorPropagator,
        }
    }
}

fn parse_effect_kind(effect: &Mapping) -> EffectKind {
    let effect = effect.keys().next().unwrap().as_str().expect("an effect must start with its name as a string.");

    effect.into()
}

fn parse_effect<T>(rng: &mut impl Rng, effect: &Mapping) -> Box<dyn Effect<T>> where
    HueRotate: Effect<T>,
    Contrast: Effect<T>,
    Brighten: Effect<T>,
    Saturate: Effect<T>,
    GradientMap: Effect<T>,
    QuantizeHue: Effect<T>,
    MultiplyHue: Effect<T>,
    Bayer: Effect<T>,
    ErrorPropagator<'static, 'static, WithPalette>: Effect<T>,
{
    let kind = parse_effect_kind(effect);

    match kind {
        EffectKind::HueRotate => Box::new(parse_hue_rotate(rng, effect)),
        EffectKind::Contrast => Box::new(parse_contrast(rng, effect)),
        EffectKind::Brighten => Box::new(parse_brighten(rng, effect)),
        EffectKind::Saturate => Box::new(parse_saturate(rng, effect)),
        EffectKind::GradientMap => Box::new(parse_gradient_map(rng, effect)),
        EffectKind::QuantizeHue => Box::new(parse_quantize_hue(rng, effect)),
        EffectKind::MultiplyHue => Box::new(parse_multiply_hue(rng, effect)),

        EffectKind::Bayer => Box::new(parse_bayer(rng, effect)),
        EffectKind::ErrorPropagator => Box::new(parse_error_propagator(rng, effect,
            effect.keys().next().unwrap().as_str().unwrap())),
    }
}

pub fn parse_effects<'a, 'b, T>(rng: &mut impl Rng, root_value: &serde_yaml::Value) -> Vec<Box<dyn Effect<T>>> where
    HueRotate: Effect<T>,
    Contrast: Effect<T>,
    Brighten: Effect<T>,
    Saturate: Effect<T>,
    GradientMap: Effect<T>,
    QuantizeHue: Effect<T>,
    MultiplyHue: Effect<T>,
    Bayer: Effect<T>,
    ErrorPropagator<'static, 'static, WithPalette>: Effect<T>,
{
    let effects = root_value.get("effects").expect("[effects] was not present - is required.")
        .as_sequence().expect("[effects] must be a list - wasn't.");

    effects.iter().enumerate().map(|(i, effect)| {
        effect.as_mapping().expect(format!("[effects.{i}] must be a map - wasn't.").as_str())
    })
        .map(|effect| {
            let keys = effect.keys().len();
            if keys != 1 {
                panic!("only one key [the effect name] is accepted by effect - found {}.", keys);
            }
            parse_effect::<T>(rng, effect)
        })
        .collect::<Vec<_>>()
}

// specific effects

fn parse_hue_rotate(rng: &mut impl Rng, effect: &Mapping) -> HueRotate {
    let param = effect.get("hue-rotate").unwrap();
    HueRotate(parse_f64_param(rng, param) as f32)
}

fn parse_contrast(rng: &mut impl Rng, effect: &Mapping) -> Contrast {
    let param = effect.get("contrast").unwrap();
    Contrast(parse_f64_param(rng, param) as f32)
}

fn parse_brighten(rng: &mut impl Rng, effect: &Mapping) -> Brighten {
    let param = effect.get("brighten").unwrap();
    Brighten(parse_f64_param(rng, param) as f32)
}

fn parse_saturate(rng: &mut impl Rng, effect: &Mapping) -> Saturate {
    let param = effect.get("saturate").unwrap();
    Saturate(parse_f64_param(rng, param) as f32)
}

fn parse_gradient_map(rng: &mut impl Rng, effect: &Mapping) -> GradientMap {
    let param = effect.get("gradient-map").unwrap();

    let map = param.as_sequence().expect("[gradient-map] must be a list of valid mappings.");
    let map = map.iter().map(|entry| {
        let entry = entry.as_mapping().expect("entries in [gradient-map] must be mappings.");
        let luma = entry.get("luma").expect("[gradient-map.?.luma] is required.")
            .as_f64().expect("[gradient-map.?.luma] must be a valid positive float.") as f32;

        let col = entry.get("colour").expect("[gradient-map.?.colour] is required.")
            .as_mapping().expect("[gradient-map.?.colour] must be a mapping.");

        (*parse_colour(rng, col).choose(rng).unwrap(), luma)
    })
    .collect::<Vec<_>>();

    GradientMap::with_map(map)
}

fn parse_quantize_hue(rng: &mut impl Rng, effect: &Mapping) -> QuantizeHue {
    let param = effect.get("quantize-hue").unwrap();

    let hues = param.as_sequence().expect("[quantize-hue] must be a list of hues/hue options.");

    let hues = hues.iter().map(|hue| parse_f64_param(rng, hue) as f32).collect::<Vec<_>>();

    QuantizeHue::with_hues(hues)
}

fn parse_multiply_hue(rng: &mut impl Rng, effect: &Mapping) -> MultiplyHue {
    let param = effect.get("multiply-hue").unwrap();
    MultiplyHue(parse_f64_param(rng, param) as f32)
}

fn parse_bayer(rng: &mut impl Rng, effect: &Mapping) -> Bayer {
    let config = effect.get("bayer").unwrap();

    let matrix_size = config.get("matrix-size").expect("[bayer] requires a [matrix-size] to be set.");
    let palette = config.get("palette").expect("[bayer] requires a [palette] to be set.");

    let matrix_size = parse_u64_param(rng, matrix_size);
    let palette = parse_palette(rng, palette);

    Bayer::new(matrix_size as usize, palette)
}

fn parse_error_propagator<'a, 'b>(rng: &mut impl Rng, effect: &Mapping, algorithm_name: &str) -> ErrorPropagator<'a, 'b, WithPalette> {
    let config = effect.get(algorithm_name).unwrap();

    let propagator = match algorithm_name.to_lowercase().as_str() {
        "floydsteinberg" | "floyd-steinberg" | "floyd_steinberg" =>
            FLOYD_STEINBERG,
        "jarvisjudiceninke" | "jarvis-judice-ninke" | "jarvis_judice_ninke" =>
            JARVIS_JUDICE_NINKE,
        "atkinson" => 
            ATKINSON,
        "burkes" =>
            BURKES,
        "stucki" =>
            STUCKI,
        "sierra" =>
            SIERRA,
        "sierra-two-row" | "sierra_two_row" =>
            SIERRA_TWO_ROW,
        "sierra-lite" | "sierra_to_row" =>
            SIERRA_LITE,
        _ => panic!("{algorithm_name} is not a supported effect."),
    };

    let palette = config.get("palette").expect(format!("[{algorithm_name}] requires a [palette] to be set.").as_str());

    let palette = parse_palette(rng, palette);

    propagator.with_palette(palette)
}

// parse common blocks

fn parse_f64_param(rng: &mut impl Rng, param: &serde_yaml::Value) -> f64 {
    if let Some(exact) = param.as_f64() {
        exact
    } else if let Some(range) = param.as_mapping() {
        let min = range.get("min")
            .expect("expected [brighten.min] due to mapping - not present.")
            .as_f64().expect("[brighten.min] must be a valid float - wasn't.");

        let max = range.get("max")
            .expect("expected [brighten.max] due to mapping - not present.")
            .as_f64().expect("[brighten.max] must be a valid float - wasn't.");

        rng.gen_range(min..max)
    } else if let Some(options) = param.as_sequence() {
        let picked = options.choose(rng).unwrap();
        let picked = picked.as_f64().expect("[brighten] options should be valid floats.");

        picked
    } else {
        todo!()
    }
}

fn parse_u64_param(rng: &mut impl Rng, param: &serde_yaml::Value) -> u64 {
    if let Some(exact) = param.as_u64() {
        exact
    } else if let Some(range) = param.as_mapping() {
        let min = range.get("min")
            .expect("expected [brighten.min] due to mapping - not present.")
            .as_u64().expect("[brighten.min] must be a valid float - wasn't.");

        let max = range.get("max")
            .expect("expected [brighten.max] due to mapping - not present.")
            .as_u64().expect("[brighten.max] must be a valid float - wasn't.");

        rng.gen_range(min..max)
    } else if let Some(options) = param.as_sequence() {
        let picked = options.choose(rng).unwrap();
        let picked = picked.as_u64().expect("[brighten] options should be valid floats.");

        picked
    } else {
        todo!()
    }
}

fn parse_palette(rng: &mut impl Rng, param: &serde_yaml::Value) -> Vec<Srgb> {
    if let Some(param) = param.as_str() {
        if param == "random" {
            generate_random_palette(rng)
        } else {
            panic!("[palette] must either be a list or 'random'.");
        }
    } else if let Some(colours) = param.as_sequence() {
        if colours.iter().any(|col| !col.is_mapping()) {
            panic!("Colours under [palette] must all be mappings / objects.");
        } else {
            colours.into_iter()
                .map(|colour| colour.as_mapping().unwrap())
                .map(|colour| parse_colour(rng, colour))
                .collect::<Vec<_>>()
                .concat()
        }
    } else {
        panic!("wuh woh");
    }
}

fn generate_random_palette(mut rng: &mut impl Rng) -> Vec<Srgb> {
    fn gen_with_lightness(rng: &mut impl Rng, min: f32, max: f32) -> Lch {
        Lch::new(
            rng.gen_range(min..=max),
            rng.gen_range(0.0..128.0),
            rng.gen_range(0.0..360.0),        
        )
    }

    let mut palette = vec![
        gen_with_lightness(rng, 80.0, 100.0),
        gen_with_lightness(rng, 20.0,  80.0),
        gen_with_lightness(rng,  0.0,  20.0),
    ];

    for _ in 0..rng.gen_range(0..10) {
        palette.push(gen_with_lightness(&mut rng,  0.0,  100.0));
    }
    
    let mut palette = palette
        .into_iter()
        .map(|col| {
            let col: Srgb = col.into_color();
            col
        })
        .map(|col| {
            if rng.gen_bool(0.10) {
                let amnt = rng.gen_range(2..=10);
                col.build_gradient_lch(amnt)
            } else {
                vec![col]
            }
        })
        .collect::<Vec<_>>()
        .concat();

    if rng.gen_bool(0.75) {
        palette.push(named::BLACK.into_format());
        palette.push(named::WHITE.into_format());
    }

    palette
}

fn parse_colour(rng: &mut impl Rng, param: &Mapping) -> Vec<Srgb> {    
    if let Some(rgb) = param.get("rgb") {
        let colour = parse_rgb(rgb);
        let gradient = if let Some(shades) = param.get("shades") {
            let shades = shades.as_u64().expect("[palette.shades] must be a positive integer.");
            colour.build_gradient_lch(shades as u16)
        } else {
            vec![colour]
        };
        gradient
    } else if let Some(amnt) = param.get("random") {
        let amnt = parse_u64_param(rng, amnt);
        let mut colours = vec![];
        for _ in 0..amnt {
            colours.push(Srgb::new(
                rng.gen_range(0.0..=1.0),
                rng.gen_range(0.0..=1.0),
                rng.gen_range(0.0..=1.0),
            ))
        }

        colours
    } else {
        panic!("wuh woh");
    }
}

fn parse_rgb(value: &serde_yaml::Value) -> Srgb {
    if let Some(value) = value.as_str() {
        if value.len() != 6 {
            panic!("Hexcodes must be 6 characters long.");
        }
        let r = u8::from_str_radix(&value[0..2], 16);
        let g = u8::from_str_radix(&value[2..4], 16);
        let b = u8::from_str_radix(&value[4..6], 16);

        if r.is_err() || g.is_err() || b.is_err() {
            panic!("{value} is not a valid hexcode.");
        }

        Srgb::new(r.unwrap(), g.unwrap(), b.unwrap()).into_format()
    } else if let Some(components) = value.as_sequence() {
        if components.len() != 3 {
            panic!("There should only be 3 RGB components. Found {}.", components.len());
        }

        if components.iter().all(|f| f.is_f64()) {
            let components = components.iter().map(|c| c.as_f64().unwrap()).collect::<Vec<_>>();
            Srgb::new(components[0] as f32, components[1] as f32, components[2] as f32)
        } else if components.iter().all(|f| f.is_u64()) {
            let components = components.iter().map(|c| c.as_u64().unwrap() as u8).collect::<Vec<_>>();
            Srgb::<u8>::new(components[0], components[1], components[2]).into_format()
        } else {
            panic!("uh oh components are bad");
        }
    } else {
        panic!("uh oh rgb is bad");
    }
}