use std::error::Error;

use image_effects::{
    dither::{
        error::{ErrorPropagator, WithPalette},
        ordered::{
            algorithms::Wrapping, DiagonalDirection, Increase, MirrorLine, Ordered,
            OrderedStrategy::*, Orientation,
        },
        ATKINSON, BURKES, FLOYD_STEINBERG, JARVIS_JUDICE_NINKE, SIERRA, SIERRA_LITE,
        SIERRA_TWO_ROW, STUCKI,
    },
    prelude::{
        filters::{Brighten, Contrast, GradientMap, HueRotate, MultiplyHue, QuantizeHue, Saturate},
        Effect, IntoGradientLch,
    },
};
use palette::{named, IntoColor, Lch, Srgb};
use rand::{seq::SliceRandom, Rng};
use serde_yaml::{Mapping, Value};

use crate::{
    logging::{alt::SystemLog, RunLog},
    parsers::{
        ordered::{
            parse_diagonaldirection, parse_increase_strategy, parse_matrix_size, parse_mirror,
            parse_orientation, parse_wrapping_set,
        },
        palette::{
            parse_chroma_strategy, parse_colour, parse_hue_strategies, parse_inject,
            parse_lum_strategy, ChromaStrategy, HueDistribution, HueStrategy, LumStrategy,
        },
        util::{
            parse_property_as_f64_complex, parse_property_as_f64_tuple_param,
            parse_property_as_str, parse_property_as_u64_complex,
            parse_value_as_f64_sequence_complex,
        },
    },
};

#[derive(Debug)]
pub enum EffectKind {
    HueRotate,
    Contrast,
    Brighten,
    Saturate,
    GradientMap,
    QuantizeHue,
    MultiplyHue,

    Ordered,
    ErrorPropagator,
}

impl From<&str> for EffectKind {
    fn from(value: &str) -> Self {
        match value {
            "hue-rotate" => Self::HueRotate,
            "contrast" => Self::Contrast,
            "brighten" => Self::Brighten,
            "saturate" => Self::Saturate,
            "gradient-map" => Self::GradientMap,
            "quantize-hue" => Self::QuantizeHue,
            "multiply-hue" => Self::MultiplyHue,
            "ordered" => Self::Ordered,
            _ => Self::ErrorPropagator,
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
    Lch::new(lum, rng.gen_range(0.0..128.0), rng.gen_range(0.0..360.0))
}

fn generate_gradient_map(
    rng: &mut impl Rng,
    amnt: u64,
    noise: f64,
    noise_chance: f64,
    min_brightness: f64,
    max_brightness: f64,
) -> Vec<(Srgb, f32)> {
    let step_size = (max_brightness - min_brightness) / ((amnt - 1) as f64);

    // must always start from the minimum brightness.
    let mut step_loc = min_brightness as f32;

    let mut palette = Vec::new();

    for _ in 0..amnt {
        palette.push((gen_with_lightness(rng, step_loc), step_loc / 100.0));
        step_loc += step_size as f32;
    }

    if noise != 0.0 && noise_chance != 0.0 {
        palette.iter_mut().for_each(|color| {
            if rng.gen_range(0.0..1.0) <= noise_chance {
                let unbounded_l = color.0.l + rng.gen_range(-noise..noise) as f32;
                color.0.l = unbounded_l.clamp(0.0, 100.0);
            }
        });
    }

    palette
        .into_iter()
        .map(|(color, step)| (color.into_color(), step))
        .collect::<Vec<_>>()
}

fn parse_effect_kind(effect: &Value) -> EffectKind {
    let effect = effect
        .as_mapping()
        .expect("whoopsies something has gone wrong")
        .keys()
        .next()
        .unwrap()
        .as_str()
        .expect("an effect must start with its name as a string.");

    effect.into()
}

fn parse_effect<T>(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<Box<dyn Effect<T>>>
where
    HueRotate: Effect<T>,
    Contrast: Effect<T>,
    Brighten: Effect<T>,
    Saturate: Effect<T>,
    GradientMap: Effect<T>,
    QuantizeHue: Effect<T>,
    MultiplyHue: Effect<T>,
    Ordered: Effect<T>,
    ErrorPropagator<'static, 'static, WithPalette>: Effect<T>,
{
    let kind = parse_effect_kind(effect);

    Ok(match kind {
        EffectKind::HueRotate => {
            log.begin_category("hue-rotate")?;
            let factor = parse_property_as_f64_complex(log, rng, effect, "hue-rotate")?
                .expect("[hue-rotate] expected as an f64.");
            log.end_category()?;
            Box::new(HueRotate(factor as f32))
        }
        EffectKind::Contrast => {
            log.begin_category("contrast")?;
            let factor = parse_property_as_f64_complex(log, rng, effect, "contrast")?
                .expect("[hue-rotate] expected as an f64.");
            log.end_category()?;
            Box::new(Contrast(factor as f32))
        }
        EffectKind::Brighten => {
            log.begin_category("brighten")?;
            let factor = parse_property_as_f64_complex(log, rng, effect, "brighten")?
                .expect("[brighten] expected as an f64.");
            log.end_category()?;
            Box::new(Brighten(factor as f32))
        }
        EffectKind::Saturate => {
            log.begin_category("saturate")?;
            let factor = parse_property_as_f64_complex(log, rng, effect, "saturate")?
                .expect("[saturate] expected as an f64.");
            log.end_category()?;
            Box::new(HueRotate(factor as f32))
        }
        EffectKind::MultiplyHue => {
            log.begin_category("multiply-hue")?;
            let factor = parse_property_as_f64_complex(log, rng, effect, "multiply-hue")?
                .expect("[multiply-hue] expected as an f64.");
            log.end_category()?;
            Box::new(HueRotate(factor as f32))
        }
        EffectKind::GradientMap => {
            log.begin_category("gradient-map")?;
            let fx = parse_gradient_map(log, rng, effect)?;
            log.end_category()?;
            Box::new(fx)
        }
        EffectKind::QuantizeHue => {
            log.begin_category("quantize-hue")?;
            let fx = parse_quantize_hue(log, rng, effect)?;
            log.end_category()?;
            Box::new(fx)
        }
        EffectKind::Ordered => {
            log.begin_category("ordered")?;
            let fx = parse_ordered(log, rng, effect)?;
            log.end_category()?;
            Box::new(fx)
        }
        EffectKind::ErrorPropagator => {
            log.begin_category("error-propagator")?;
            log.alert("error propagator not currently being logged.")?;
            let fx = parse_error_propagator(
                log,
                rng,
                effect,
                effect
                    .as_mapping()
                    .expect("this effect needs to be a mapping you bastard")
                    .keys()
                    .next()
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )?;
            log.end_category();
            Box::new(fx)
        }
    })
}

pub type Log<'a> = &'a mut SystemLog;
pub type BaseResult<T> = Result<T, Box<dyn Error>>;

pub fn parse_effects<'a, 'b, T>(
    log: Log,
    rng: &mut impl Rng,
    root_value: &serde_yaml::Value,
) -> BaseResult<Vec<Box<dyn Effect<T>>>>
where
    HueRotate: Effect<T>,
    Contrast: Effect<T>,
    Brighten: Effect<T>,
    Saturate: Effect<T>,
    GradientMap: Effect<T>,
    QuantizeHue: Effect<T>,
    MultiplyHue: Effect<T>,
    Ordered: Effect<T>,
    ErrorPropagator<'static, 'static, WithPalette>: Effect<T>,
{
    let effects = root_value
        .get("effects")
        .expect("[effects] was not present - is required.")
        .as_sequence()
        .expect("[effects] must be a list - wasn't.");

    Ok(effects
        .iter()
        .enumerate()
        .map(|(i, effect)| {
            effect
                .as_mapping()
                .unwrap_or_else(|| panic!("[effects.{i}] must be a map - wasn't."))
        })
        .map(|effect| {
            let keys = effect.keys().len();
            if keys != 1 {
                panic!("only one key [the effect name] is accepted by effect - found {keys}.");
            }
            parse_effect::<T>(log, rng, &Value::Mapping(effect.clone()))
                .expect("failure when parsing effect. please check log.")
        })
        .collect::<Vec<_>>())
}

// specific effects

fn parse_hue_rotate(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<HueRotate> {
    let degrees = parse_property_as_f64_complex(log, rng, effect, "degrees")?
        .expect("expected [degrees] as f64") as f32;
    Ok(HueRotate(degrees))
}

fn parse_contrast(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<Contrast> {
    let degrees = parse_property_as_f64_complex(log, rng, effect, "contrast")?
        .expect("expected [contrast] as f64") as f32;
    Ok(Contrast(degrees))
}

fn parse_brighten(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<Brighten> {
    let factor = parse_property_as_f64_complex(log, rng, effect, "brighten")?
        .expect("expected [brighten] as f64") as f32;
    Ok(Brighten(factor))
}

fn parse_saturate(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<Saturate> {
    let param = effect.get("saturate").unwrap();

    let factor = parse_property_as_f64_complex(log, rng, effect, "saturate")?
        .expect("expected [saturate] as f64") as f32;
    Ok(Saturate(factor))
}

fn parse_gradient_map(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<GradientMap> {
    let param = effect.get("gradient-map").unwrap();

    Ok(if param.is_mapping() {
        let amnt = parse_property_as_u64_complex(log, rng, param, "amnt")?
            .expect("[gradient-map] as a mapping needs an [amnt] property.");
        let noise = parse_property_as_f64_complex(log, rng, param, "noise")?.unwrap_or(0.0);
        let noise_chance =
            parse_property_as_f64_complex(log, rng, param, "noise-chance")?.unwrap_or(1.0);
        let min_brightness =
            parse_property_as_f64_complex(log, rng, param, "min-brightness")?.unwrap_or(0.0);
        let max_brightness =
            parse_property_as_f64_complex(log, rng, param, "max-brightness")?.unwrap_or(100.0);

        let generated_map = generate_gradient_map(
            rng,
            amnt,
            noise,
            noise_chance,
            min_brightness,
            max_brightness,
        );

        GradientMap::with_map(generated_map)
    } else {
        log.alert("too tired. don't wanna do gradient map right now. sry.");

        let map = param
            .as_sequence()
            .expect("[gradient-map] must be a list of valid mappings (or a mapping itself).");
        let map = map
            .iter()
            .map(|entry| {
                let entry = entry
                    .as_mapping()
                    .expect("entries in [gradient-map] must be mappings.");
                let luma = entry
                    .get("luma")
                    .expect("[gradient-map.?.luma] is required.")
                    .as_f64()
                    .expect("[gradient-map.?.luma] must be a valid positive float.")
                    as f32;

                let col = entry
                    .get("colour")
                    .expect("[gradient-map.?.colour] is required.")
                    .as_mapping()
                    .expect("[gradient-map.?.colour] must be a mapping.");

                // (*parse_colour(log, rng, col)?.choose(rng).unwrap(), luma)
                panic!("seems like you tried to make a gradient map! whoops! no way! we're doing a refactor and this thing is FUCKED!");
            })
            .collect::<Vec<_>>();

        GradientMap::with_map(map)
    })
}

fn parse_quantize_hue(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<QuantizeHue> {
    let param = effect.get("quantize-hue").unwrap();

    let hues = parse_value_as_f64_sequence_complex(log, rng, param, "hues")?
        .expect("[quantize-hue.hues] must be a list of hues/hue options.");

    let hues = hues.iter().map(|h| *h as f32).collect::<Vec<_>>();

    let hues_str = hues
        .iter()
        .enumerate()
        .map(|h| (h.0, format!("{:.3}", h.1)))
        .collect::<Vec<_>>();

    log.state_property("hues", format!("{hues_str:?}").as_str())?;

    Ok(QuantizeHue::with_hues(hues))
}

fn parse_multiply_hue(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<MultiplyHue> {
    let factor = parse_property_as_f64_complex(log, rng, effect, "multiply-hue")?
        .expect("expected [multiply-hue] as a f64") as f32;

    Ok(MultiplyHue(factor))
}

fn parse_ordered(log: Log, rng: &mut impl Rng, effect: &Value) -> BaseResult<Ordered> {
    let config = effect.get("ordered").unwrap();

    let strategy = &parse_property_as_str(log, config, "strategy")?
        .expect("[ordered] requires a [strategy] to be set as a string.");

    let invert_chance = parse_property_as_f64_complex(log, rng, config, "invert")?.unwrap_or(0.0);

    let palette = config
        .get("palette")
        .expect("[strategy] requires a [palette] to be set.");
    let palette = parse_palette(log, rng, palette)?;

    log.begin_category("palette")?;
    for (i, col) in palette.iter().enumerate() {
        log.state_property(
            format!("#{i:03}"),
            format!("RGB: ({:3.3},{:3.3},{:3.3})", col.red, col.green, col.blue),
        )?;
    }
    log.end_category()?;

    let mirror = config.get("mirror").map(|mirror| {
        log.begin_category("mirror").expect("fucked up when beginning the mirror category for some reason");
        let mirror = parse_mirror(log, rng, mirror).expect("okay yeah if you see this, something fucked up in the mirrors. good luck. just look for the code [X93HC] and you'll find this in the source code.");
        log.end_category().expect("fucked up when ending the category for some reason");
        mirror
    });

    log.begin_category(strategy)?;
    let mut strategy = match strategy.as_str() {
        "bayer" => {
            let size = parse_matrix_size(log, rng, config, strategy)? as usize;

            Bayer(size)
        }
        "diamonds" => {
            let size = parse_matrix_size(log, rng, config, strategy)? as usize;
            Diamonds(size)
        }
        "checkered-diamonds" => {
            let size = parse_matrix_size(log, rng, config, strategy)? as usize;
            CheckeredDiamonds(size)
        }
        "stars" => Stars,
        "new-stars" => NewStars,
        "grid" => Grid,
        "trail" => Trail,
        "criss-cross" => Crisscross,
        "static" => Static,
        "wavy" => {
            let orientation = parse_orientation(log, rng, config, strategy)?;
            let o_str = match &orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            };
            Wavy(orientation)
        }
        "bootleg-bayer" => BootlegBayer,
        "diagonals" => Diagonals,
        "diagonals-big" => DiagonalsBig,
        "diamond-grid" => DiamondGrid,
        "speckle-squares" => SpeckleSquares,
        "scales" => Scales,
        "trail-scales" => TrailScales,
        "diagonals-n" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;
            let direction = parse_diagonaldirection(log, rng, config, strategy)?;
            let increase = parse_increase_strategy(log, rng, config, strategy)?;

            DiagonalsN {
                n,
                direction: direction.clone(),
                increase: increase.clone(),
            }
        }
        "diagonal-tiles" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;

            DiagonalTiles(n)
        }
        "bouncing-bowtie" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;

            BouncingBowtie(n)
        }
        "scanline" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;
            let orientation = parse_orientation(log, rng, config, strategy);

            ScanLine(n, orientation?.clone())
        }
        "starburst" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;

            Starburst(n)
        }
        "shiny-bowtie" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;

            ShinyBowtie(n)
        }
        "marble-tile" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;

            MarbleTile(n)
        }
        "curve-path" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;
            let amplitude =
                parse_property_as_f64_complex(log, rng, config, "amplitude")?.unwrap_or(1.0);
            let promotion =
                parse_property_as_f64_complex(log, rng, config, "promotion")?.unwrap_or(0.0);
            let halt_threshold = parse_property_as_u64_complex(log, rng, config, "halt-threshold")?
                .unwrap_or(100) as usize;

            CurvePath {
                n,
                amplitude,
                promotion,
                halt_threshold,
            }
        }
        "zigzag" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;
            let halt_threshold = parse_property_as_u64_complex(log, rng, config, "halt-threshold")?
                .unwrap_or(100) as usize;
            let wrapping = parse_wrapping_set(log, rng, config)?
                .choose(rng)
                .unwrap()
                .clone();

            let magnitude =
                parse_property_as_f64_tuple_param(log, rng, config, "magnitude", ("y", "x"))?;
            let promotion =
                parse_property_as_f64_tuple_param(log, rng, config, "promotion", ("y", "x"))?;

            let magnitude = (magnitude.0.unwrap_or(1.0), magnitude.1.unwrap_or(1.0));
            let promotion = (promotion.0.unwrap_or(0.0), promotion.1.unwrap_or(0.0));

            ZigZag {
                n,
                halt_threshold,
                wrapping: wrapping.clone(),
                magnitude,
                promotion,
            }
        }
        "broken-spiral" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;

            let base_step =
                parse_property_as_f64_tuple_param(log, rng, config, "base-step", ("y", "x"))?;
            let base_step = (base_step.0.unwrap_or(1.0), base_step.1.unwrap_or(1.0));

            let oob_threshold = parse_property_as_u64_complex(log, rng, config, "oob-threshold")?
                .unwrap_or((n as f64 / (base_step.0.min(base_step.1))) as u64)
                as usize;
            let increment_by =
                parse_property_as_f64_complex(log, rng, config, "increment-by")?.unwrap_or(0.0);
            let increment_in = parse_property_as_u64_complex(log, rng, config, "increment-in")?
                .unwrap_or(1) as usize;

            let n = parse_matrix_size(log, rng, config, strategy)? as usize;

            BrokenSpiral {
                n,
                base_step,
                oob_threshold,
                increment_by,
                increment_in,
            }
        }
        "modulo-snake" => {
            let n = parse_matrix_size(log, rng, config, strategy)? as usize;
            let increment_by =
                parse_property_as_f64_complex(log, rng, config, "increment-by")?.unwrap_or(1.0);
            let modulo =
                parse_property_as_u64_complex(log, rng, config, "modulo")?.unwrap_or(10) as usize;
            let iterations = parse_property_as_u64_complex(log, rng, config, "iterations")?
                .unwrap_or(1) as usize;

            ModuloSnake {
                n,
                increment_by,
                modulo,
                iterations,
            }
        }
        _ => {
            let strategies = vec![
                "bayer",
                "diamonds",
                "checkered-diamonds",
                "stars",
                "new-stars",
                "grid",
                "trail",
                "criss-cross",
                "static",
                "wavy",
                "bootleg-bayer",
                "diagonals",
                "diagonals-big",
                "diamond-grid",
                "speckle-squares",
                "scales",
                "trail-scales",
                "diagonals-n",
                "diagonal-tiles",
                "bouncing-bowtie",
                "scanline",
                "starburst",
                "shiny-bowtie",
                "marble-tile",
                "curve-path",
                "broken-spiral",
                "modulo-snake",
            ];
            panic!("{strategy} is an invalid [ordered.strategy]. Allowed strategies are: {strategies:?}");
        }
    };
    log.end_category()?;

    if let Some((chance, mirror_sets)) = mirror {
        if rng.gen_range(0.0..1.0) <= chance {
            for mirrorline in mirror_sets {
                strategy = strategy.mirror(mirrorline);
            }
        };
    }

    strategy = if rng.gen_range(0.0..1.0) <= invert_chance {
        strategy.invert()
    } else {
        strategy
    };

    Ok(Ordered::new(palette, strategy))
}

fn parse_error_propagator<'a, 'b>(
    log: Log,
    rng: &mut impl Rng,
    effect: &Value,
    algorithm_name: &str,
) -> BaseResult<ErrorPropagator<'a, 'b, WithPalette>> {
    let config = effect.get(algorithm_name).unwrap();

    let propagator = match algorithm_name.to_lowercase().as_str() {
        "floydsteinberg" | "floyd-steinberg" | "floyd_steinberg" => FLOYD_STEINBERG,
        "jarvisjudiceninke" | "jarvis-judice-ninke" | "jarvis_judice_ninke" => JARVIS_JUDICE_NINKE,
        "atkinson" => ATKINSON,
        "burkes" => BURKES,
        "stucki" => STUCKI,
        "sierra" => SIERRA,
        "sierra-two-row" | "sierra_two_row" => SIERRA_TWO_ROW,
        "sierra-lite" | "sierra_to_row" => SIERRA_LITE,
        _ => panic!("{algorithm_name} is not a supported effect."),
    };

    let palette = config
        .get("palette")
        .unwrap_or_else(|| panic!("[{algorithm_name}] requires a [palette] to be set."));

    let palette = parse_palette(log, rng, palette)?;

    Ok(propagator.with_palette(palette))
}

// parse common blocks

fn parse_palette(log: Log, rng: &mut impl Rng, param: &serde_yaml::Value) -> BaseResult<Vec<Srgb>> {
    log.pause();
    log.alert("PARSE PALETTE is unsupported for now")?;
    let palette = if let Some(palette) = param.as_mapping() {
        let palette_type = palette
            .get("type")
            .expect("[palette] requires a [.type] to be specified.")
            .as_str()
            .expect("[palette.type] must be a string.");

        Ok(match palette_type {
            "random_v1" => generate_random_palette(rng),
            "specified" => {
                let colours = palette
                    .get("colours")
                    .expect(
                        "if [palette.type] is \"specified\", [palette.colours] must be present.",
                    )
                    .as_sequence()
                    .expect("[palette.colours] must be a list of valid colours");

                colours
                    .iter()
                    .map(|colour| parse_colour(log, rng, colour).expect("yeah i mean you used a specified palette and the colours are fucked up in some way. what can i say. im tired. ive been at this for hours now."))
                    .collect::<Vec<_>>()
                    .concat()
            }
            "random_v2" => {
                let config = palette.get("config").expect(
                    "if [palette.type] is \"random_v2\", [palette.config] must be present.",
                );

                generate_random_palette_v2(log, rng, config)?
            }
            _ => {
                panic!("{palette_type} is not a valid palette type.");
            }
        })
    } else {
        panic!("wuh woh");
    };
    log.unpause();
    palette
}

fn generate_random_palette(mut rng: &mut impl Rng) -> Vec<Srgb> {
    let mut palette = vec![
        gen_with_random_lightness(rng, 80.0, 100.0),
        gen_with_random_lightness(rng, 20.0, 80.0),
        gen_with_random_lightness(rng, 0.0, 20.0),
    ];

    for _ in 0..rng.gen_range(0..10) {
        palette.push(gen_with_random_lightness(&mut rng, 0.0, 100.0));
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

fn generate_random_palette_v2(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<Vec<Srgb>> {
    let max_lum = parse_property_as_f64_complex(log, rng, value, "max-lum")?.unwrap_or(100.0);
    let min_lum = parse_property_as_f64_complex(log, rng, value, "min-lum")?.unwrap_or(0.0);

    log.begin_category("lum-strategy")?;
    let (lum_strategy, lum_amnt) = parse_lum_strategy(log, rng, value)?;
    log.end_category()?;

    let inject = parse_inject(log, rng, value);

    log.begin_category("hue-strategies")?;
    let hue_strategies = parse_hue_strategies(log, rng, value)?;
    log.end_category()?;

    log.begin_category("chroma-strategy")?;
    let chroma_strategy = parse_chroma_strategy(log, rng, value)?;
    log.end_category()?;

    let misc_flags = value.get("misc_flags").map(|param| {
        param
            .as_sequence()
            .expect("[palette.config.misc_flags] must be a list")
            .iter()
            .map(|param| {
                param
                    .as_str()
                    .expect("[palette.config.misc_flags] must be a list of strings.")
            })
            .collect::<Vec<&str>>()
    });

    let mut flag_lum_safeguard = false;
    let mut flag_extremes = false;
    let mut flag_single_lum = false;

    if let Some(flags) = misc_flags {
        if flags.contains(&"lum_safeguard") {
            flag_lum_safeguard = true
        }
        if flags.contains(&"extremes") {
            flag_extremes = true
        }
        if flags.contains(&"single_lum") {
            flag_single_lum = true
        }
    };

    let mut palette: Vec<Lch> = Vec::new();

    let seed_hue = rng.gen_range(0.0..360.0);
    let mut hues = vec![seed_hue];

    // fn for common hue calculation
    let mut generate_hue_neighbourhood = |hue: f64, size: f64, n: u64, dist: &HueDistribution| {
        let mut neighbourhood = Vec::new();

        let lower_end = hue - size;
        let upper_end = hue + size;

        for i in 0..n {
            match dist {
                HueDistribution::Linear => {
                    let fraction = (i as f64) / ((n - 1) as f64);
                    neighbourhood.push(lower_end + (size * 2.0 * fraction));
                }
                HueDistribution::Random => {
                    neighbourhood.push(rng.gen_range(lower_end..upper_end));
                }
            }
        }

        neighbourhood
    };

    // hue strategy application
    for strategy in hue_strategies.iter() {
        match strategy {
            HueStrategy::Neighbour { size, n, dist } => {
                hues = [hues, generate_hue_neighbourhood(seed_hue, *size, *n, dist)].concat()
            }
            HueStrategy::Contrast { size, n, dist } => {
                hues = [
                    hues,
                    generate_hue_neighbourhood(seed_hue + 180.0, *size, *n, dist),
                ]
                .concat()
            }
            HueStrategy::Penpal {
                size,
                n,
                dist,
                distance,
            } => {
                hues = [
                    hues,
                    generate_hue_neighbourhood(seed_hue + distance, *size, *n, dist),
                ]
                .concat()
            }
            HueStrategy::Cycle { n } => {
                for i in 1..=*n {
                    hues.push(seed_hue + i as f64 * (360.0 / (*n as f64 + 1.0)));
                }
            }
        }
    }

    // lum strategy application
    hues.into_iter().for_each(|hue| {
        let hue = hue as f32;

        match &lum_strategy {
            LumStrategy::Exact(lums) => {
                for mut l in lums.iter() {
                    if flag_single_lum {
                        l = lums.choose(rng).unwrap()
                    };
                    palette.push(Lch::new(*l as f32, rng.gen_range(0.0..128.0), hue));
                    if flag_single_lum {
                        break;
                    };
                }
            }
            LumStrategy::Random { unified: _ } => {
                for _ in 0..lum_amnt {
                    palette.push(Lch::new(
                        rng.gen_range(0.0..100.0),
                        rng.gen_range(0.0..128.0),
                        hue,
                    ));
                    if flag_single_lum {
                        break;
                    };
                }
            }
            LumStrategy::Distributed => {
                for mut i in 0..lum_amnt {
                    if flag_single_lum {
                        i = rng.gen_range(0..lum_amnt)
                    };
                    let span_size = max_lum - min_lum;
                    let l = min_lum + (i as f64 / (lum_amnt as f64 - 1.0)) * span_size;
                    palette.push(Lch::new(l as f32, rng.gen_range(0.0..128.0), hue));
                    if flag_single_lum {
                        break;
                    };
                }
            }
            LumStrategy::DistributedArea { overlap } => {
                for mut i in 0..lum_amnt {
                    if flag_single_lum {
                        i = rng.gen_range(0..lum_amnt)
                    };
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
                    if flag_single_lum {
                        break;
                    };
                }
            }
            LumStrategy::DistributedNudge { nudge_size } => {
                for mut i in 0..lum_amnt {
                    if flag_single_lum {
                        i = rng.gen_range(0..lum_amnt)
                    };
                    let span_size = max_lum - min_lum;
                    let mut l = min_lum + (i as f64 / (lum_amnt as f64 - 1.0)) * span_size;

                    l = (l + rng.gen_range((-nudge_size)..*nudge_size)).clamp(0.0, 100.0);

                    palette.push(Lch::new(l as f32, rng.gen_range(0.0..128.0), hue));
                    if flag_single_lum {
                        break;
                    };
                }
            }
        }
    });

    match chroma_strategy {
        ChromaStrategy::Random(range) => {
            palette
                .iter_mut()
                .for_each(|col| col.chroma = rng.gen_range(range.clone()) as f32);
        }
    }

    // injection
    if flag_lum_safeguard {
        palette.push(gen_with_random_lightness(rng, 80.0, 100.0));
        palette.push(gen_with_random_lightness(rng, 20.0, 80.0));
        palette.push(gen_with_random_lightness(rng, 0.0, 20.0));
    }

    if flag_extremes {
        palette.push(Lch::new(0.0, 0.0, 0.0));
        palette.push(Lch::new(100.0, 128.0, 0.0));
    }

    if let Some(colours) = inject {
        let lch_colours: Vec<Lch> = colours
            .into_iter()
            .map(|colour| colour.into_color())
            .collect();
        palette = [palette, lch_colours].concat();
    }

    Ok(palette
        .into_iter()
        .map(|color| color.into_color())
        .collect())
}
