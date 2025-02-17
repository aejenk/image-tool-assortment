use std::collections::HashMap;

use image_effects::{prelude::{Effect, filters::{Brighten, HueRotate, Contrast, Saturate, GradientMap, QuantizeHue, MultiplyHue}, IntoGradientLch}, dither::{bayer::Bayer, error::{ErrorPropagator, WithPalette}, ATKINSON, FLOYD_STEINBERG, JARVIS_JUDICE_NINKE, BURKES, STUCKI, SIERRA, SIERRA_TWO_ROW, SIERRA_LITE}};
use palette::{Srgb, Lch, IntoColor, named};
use rand::{Rng, seq::SliceRandom};
use serde_yaml::{Mapping, Sequence};

use crate::parsers::{palette::{parse_colour, parse_hue_strategies, parse_inject, parse_lum_strategy, HueDistribution, HueStrategy, LumStrategy}, parse_f64_param, parse_u64_param};

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

fn gen_with_random_lightness(rng: &mut impl Rng, min: f32, max: f32) -> Lch {
    Lch::new(
        rng.gen_range(min..=max),
        rng.gen_range(0.0..128.0),
        rng.gen_range(0.0..360.0),        
    )
}

fn gen_with_lightness(rng: &mut impl Rng, lum: f32) -> Lch {
    Lch::new(
        lum,
        rng.gen_range(0.0..128.0),
        rng.gen_range(0.0..360.0),        
    )
}

fn generate_gradient_map(rng: &mut impl Rng, amnt: u64, noise: f64, noise_chance: f64, min_brightness: f64, max_brightness: f64) -> Vec<(Srgb, f32)> {
    let step_size = (max_brightness - min_brightness) / ((amnt - 1) as f64);

    // must always start from the minimum brightness.
    let mut step_loc = min_brightness as f32;

    let mut palette = Vec::new();

    for i in 0..amnt {
        palette.push((gen_with_lightness(rng, step_loc), step_loc/100.0));
        step_loc = step_loc + step_size as f32;
    }

    if noise != 0.0 && noise_chance != 0.0 {
        palette.iter_mut().for_each(|color| {
            if rng.gen_range(0.0..1.0) <= noise_chance {
                let unbounded_l = color.0.l + rng.gen_range((noise*-1.0)..noise) as f32;
                color.0.l = unbounded_l.min(100.0).max(0.0);
            }
        });
    }

    palette.into_iter().map(|(color, step)| (color.into_color(), step)).collect::<Vec<_>>()
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

    let degrees = parse_f64_param(rng, param) as f32;
    HueRotate(degrees)
}

fn parse_contrast(rng: &mut impl Rng, effect: &Mapping) -> Contrast {
    let param = effect.get("contrast").unwrap();

    let factor = parse_f64_param(rng, param) as f32;
    Contrast(factor)
}

fn parse_brighten(rng: &mut impl Rng, effect: &Mapping) -> Brighten {
    let param = effect.get("brighten").unwrap();

    let factor = parse_f64_param(rng, param) as f32;
    Brighten(factor)
}

fn parse_saturate(rng: &mut impl Rng, effect: &Mapping) -> Saturate {
    let param = effect.get("saturate").unwrap();

    let factor = parse_f64_param(rng, param) as f32;
    Saturate(factor)
}

fn parse_gradient_map(rng: &mut impl Rng, effect: &Mapping) -> GradientMap {
    let param = effect.get("gradient-map").unwrap();

    if let Some(mapping) = param.as_mapping() {
        let amnt = mapping
            .get("amnt")
            .map(|param| parse_u64_param(rng, param))
            .expect("If [gradient-map] is a mapping, it must have an [amnt] property.");

        // TODO: Turn into a mapping that can either be "type: brightness" or "type: index".
        let noise = mapping
            .get("noise")
            .map_or(0.0,|factor| parse_f64_param(rng, factor));

        let noise_chance = mapping
            .get("noise-chance")
            .map_or(1.0,|factor| parse_f64_param(rng, factor));

        let min_brightness = mapping
            .get("min-brightness")
            .map_or(0.0, |param| parse_f64_param(rng, param));

        let max_brightness = mapping
            .get("max-brightness")
            .map_or(100.0, |param| parse_f64_param(rng, param));


        let generated_map = generate_gradient_map(
            rng,
            amnt, 
            noise,
            noise_chance,
            min_brightness,
            max_brightness,
        );

        return GradientMap::with_map(generated_map);
    }

    let map = param.as_sequence().expect("[gradient-map] must be a list of valid mappings (or a mapping itself).");
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

    let factor = parse_f64_param(rng, param) as f32;
    MultiplyHue(factor)
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

fn parse_palette(rng: &mut impl Rng, param: &serde_yaml::Value) -> Vec<Srgb> {
    if let Some(palette) = param.as_mapping() {
        let palette_type = palette
            .get("type").expect("[palette] requires a [.type] to be specified.")
            .as_str().expect("[palette.type] must be a string.");

        match palette_type {
            "random_v1" => generate_random_palette(rng),
            "specified" => {
                let colours = palette
                    .get("colours").expect("if [palette.type] is \"specified\", [palette.colours] must be present.")
                    .as_sequence().expect("[palette.colours] must be a list of valid colours");

                colours.into_iter()
                    .map(|colour| colour.as_mapping().unwrap())
                    .map(|colour| parse_colour(rng, colour))
                    .collect::<Vec<_>>()
                    .concat()
            },
            "random_v2" => {
                let config = palette
                    .get("config").expect("if [palette.type] is \"random_v2\", [palette.config] must be present.")
                    .as_mapping().expect("[palette.config] must be a mapping with valid options.");

                generate_random_palette_v2(rng, config)
            },
            _ => {
                panic!("{palette_type} is not a valid palette type.");
            }
        }
    } else {
        panic!("wuh woh");
    }
}

fn generate_random_palette(mut rng: &mut impl Rng) -> Vec<Srgb> {
    let mut palette = vec![
        gen_with_random_lightness(rng, 80.0, 100.0),
        gen_with_random_lightness(rng, 20.0,  80.0),
        gen_with_random_lightness(rng,  0.0,  20.0),
    ];

    for _ in 0..rng.gen_range(0..10) {
        palette.push(gen_with_random_lightness(&mut rng,  0.0,  100.0));
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

fn generate_random_palette_v2(mut rng: &mut impl Rng, config: &Mapping) -> Vec<Srgb> {
    let max_lum = config.get("max_lum").map_or(100.0, |param| parse_f64_param(rng, param));

    let min_lum = config.get("min_lum").map_or(0.0, |param| parse_f64_param(rng, param));

    let (lum_strategy, lum_amnt) = parse_lum_strategy(rng, config);

    let inject = parse_inject(rng, config);

    let hue_strategies = parse_hue_strategies(rng, config);

    let misc_flags = config.get("misc_flags")
        .map(|param| param
            .as_sequence().expect("[palette.config.misc_flags] must be a list")
            .iter().map(|param| param.as_str().expect("[palette.config.misc_flags] must be a list of strings."))
            .collect::<Vec<&str>>());

    let mut flag_grayscale = false;
    let mut flag_lum_safeguard = false;
    let mut flag_extremes = false;

    if let Some(flags) = misc_flags {
        if flags.contains(&"grayscale") { flag_grayscale = true }
        if flags.contains(&"lum_safeguard") { flag_lum_safeguard = true }
        if flags.contains(&"extremes") { flag_extremes = true }
    };

    let mut palette: Vec<Lch> = if let Some(colours) = inject {
        colours.into_iter().map(|colour| colour.into_color()).collect()
    } else {
        Vec::new()
    };

    if flag_lum_safeguard {
        palette.push(gen_with_random_lightness(rng, 80.0, 100.0));
        palette.push(gen_with_random_lightness(rng, 20.0,  80.0));
        palette.push(gen_with_random_lightness(rng,  0.0,  20.0));
    }

    let seed_hue = rng.gen_range(0.0..360.0);
    let mut hues = vec![seed_hue];

    let mut generate_hue_neighbourhood = |hue: f64, size: f64, n: u64, dist: &HueDistribution| {
        let mut neighbourhood = Vec::new();

        let lower_end = hue - size;
        let upper_end = hue + size;

        for i in 0..n {
            match dist {
                HueDistribution::Linear => {
                    let fraction = (i as f64) / ((n-1) as f64);
                    neighbourhood.push(lower_end + (size * 2.0 * fraction));
                },
                HueDistribution::Random => {
                    neighbourhood.push(rng.gen_range(lower_end..upper_end));
                },
            }
        }

        neighbourhood
    };

    for strategy in hue_strategies.iter() {
        match strategy {
            HueStrategy::Neighbour { size, n, dist } => {
                hues = vec![hues, generate_hue_neighbourhood(seed_hue, *size, *n, dist)].concat()
            },
            HueStrategy::Contrast { size, n, dist } => {
                hues = vec![hues, generate_hue_neighbourhood(seed_hue + 180.0, *size, *n, dist)].concat()
            },
            HueStrategy::Cycle { n } => {
                for i in 1..=*n {
                    hues.push(seed_hue + i as f64 * (360.0/ (*n as f64 +1.0)));
                }
            }
        }
    }

    hues.into_iter().for_each(|hue| {
        let hue = hue as f32;
        let mut get_chroma = || rng.gen_range(0.0..128.0);

        match &lum_strategy {
            LumStrategy::Exact(lums) => {
                for l in lums.iter() {
                    palette.push(Lch::new(*l as f32, rng.gen_range(0.0..128.0), hue));
                }
            },
            LumStrategy::Random { unified } => {
                for i in 0..lum_amnt {
                    palette.push(Lch::new(rng.gen_range(0.0..100.0), rng.gen_range(0.0..128.0), hue));
                }
            },
            LumStrategy::Distributed => {
                for i in 0..lum_amnt {
                    let span_size = max_lum - min_lum;
                    let l = min_lum + (i as f64 / (lum_amnt as f64-1.0)) * span_size;
                    palette.push(Lch::new(l as f32, rng.gen_range(0.0..128.0), hue));
                }
            },
            LumStrategy::DistributedArea { overlap } => {
                for i in 0..lum_amnt {
                    let span_size = max_lum - min_lum;
                    let step_size = span_size / lum_amnt as f64;

                    let mut area_start = min_lum + (i as f64 * step_size);
                    let mut area_end = area_start + step_size;

                    if let Some(overlap) = overlap {
                        area_start = (area_start - overlap).max(min_lum);
                        area_end = (area_end + overlap).min(max_lum);
                    }

                    let l = rng.gen_range(area_start..area_end) as f32;
                    palette.push(Lch::new(l, rng.gen_range(0.0..128.0), hue));
                }
            },
            LumStrategy::DistributedNudge { nudge_size } => {
                for i in 0..lum_amnt {
                    let span_size = max_lum - min_lum;
                    let mut l = min_lum + (i as f64 / (lum_amnt as f64-1.0)) * span_size;

                    l = (l + rng.gen_range((-1.0 * nudge_size)..*nudge_size)).max(100.0).min(0.0);

                    palette.push(Lch::new(l as f32, rng.gen_range(0.0..128.0), hue));
                }
            }
        }
    });

    if flag_extremes {
        palette.push(Lch::new(0.0, 0.0, 0.0));
        palette.push(Lch::new(100.0, 128.0, 0.0));
    }

    palette.into_iter().map(|color| color.into_color()).collect()
}