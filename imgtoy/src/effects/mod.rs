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
use serde_yaml::Mapping;

use crate::{
    logging::RunLog,
    parsers::{
        ordered::{
            parse_diagonaldirection, parse_increase_strategy, parse_matrix_size, parse_mirror,
            parse_orientation, parse_wrapping_set,
        },
        palette::{
            parse_chroma_strategy, parse_colour, parse_hue_strategies, parse_inject,
            parse_lum_strategy, ChromaStrategy, HueDistribution, HueStrategy, LumStrategy,
        },
        parse_f64_param, parse_u64_param,
        util::{
            parse_property_as_f64_param, parse_property_as_f64_tuple_param,
            parse_property_as_str_param, parse_property_as_u64_param,
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

fn parse_effect_kind(effect: &Mapping) -> EffectKind {
    let effect = effect
        .keys()
        .next()
        .unwrap()
        .as_str()
        .expect("an effect must start with its name as a string.");

    effect.into()
}

fn parse_effect<T>(log: Log, rng: &mut impl Rng, effect: &Mapping) -> Box<dyn Effect<T>>
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

    match kind {
        EffectKind::HueRotate => {
            let fx = parse_hue_rotate(rng, effect);
            log.apply_effect("hue-rotate", vec![("shift", format!("{}", fx.0))]);
            Box::new(fx)
        }
        EffectKind::Contrast => {
            let fx = parse_contrast(rng, effect);
            log.apply_effect("contrast", vec![("factor", format!("{}", fx.0))]);
            Box::new(fx)
        }
        EffectKind::Brighten => {
            let fx = parse_brighten(rng, effect);
            log.apply_effect("brighten", vec![("factor", format!("{}", fx.0))]);
            Box::new(fx)
        }
        EffectKind::Saturate => {
            let fx = parse_saturate(rng, effect);
            log.apply_effect("saturate", vec![("factor", format!("{}", fx.0))]);
            Box::new(fx)
        }
        EffectKind::MultiplyHue => {
            let fx = parse_multiply_hue(log, rng, effect);
            Box::new(fx)
        }
        EffectKind::GradientMap => {
            let fx = parse_gradient_map(log, rng, effect);
            Box::new(fx)
        }
        EffectKind::QuantizeHue => {
            let fx = parse_quantize_hue(log, rng, effect);
            Box::new(fx)
        }
        EffectKind::Ordered => {
            let fx = parse_ordered(log, rng, effect);
            Box::new(fx)
        }

        EffectKind::ErrorPropagator => {
            let fx = parse_error_propagator(
                rng,
                effect,
                effect.keys().next().unwrap().as_str().unwrap(),
            );
            Box::new(fx)
        }
    }
}

type Log<'a> = &'a mut RunLog;

pub fn parse_effects<'a, 'b, T>(
    log: Log,
    rng: &mut impl Rng,
    root_value: &serde_yaml::Value,
) -> Vec<Box<dyn Effect<T>>>
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

    effects
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
            parse_effect::<T>(log, rng, effect)
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

fn parse_gradient_map(log: Log, rng: &mut impl Rng, effect: &Mapping) -> GradientMap {
    let param = effect.get("gradient-map").unwrap();

    if let Some(mapping) = param.as_mapping() {
        let amnt = mapping
            .get("amnt")
            .map(|param| parse_u64_param(rng, param))
            .expect("If [gradient-map] is a mapping, it must have an [amnt] property.");

        // TODO: Turn into a mapping that can either be "type: brightness" or "type: index".
        let noise = mapping
            .get("noise")
            .map_or(0.0, |factor| parse_f64_param(rng, factor));

        let noise_chance = mapping
            .get("noise-chance")
            .map_or(1.0, |factor| parse_f64_param(rng, factor));

        let min_brightness = mapping
            .get("min-brightness")
            .map_or(0.0, |param| parse_f64_param(rng, param));

        let max_brightness = mapping
            .get("max-brightness")
            .map_or(100.0, |param| parse_f64_param(rng, param));

        log.apply_effect("gradient-map", vec![("noise", format!("{noise}"))]);
        log.apply_effect(
            "gradient-map",
            vec![("noise-chance", format!("{}", noise_chance))],
        );
        log.apply_effect(
            "gradient-map",
            vec![("min-brightness", format!("{}", min_brightness))],
        );
        log.apply_effect(
            "gradient-map",
            vec![("max-brightness", format!("{}", max_brightness))],
        );

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

            (*parse_colour(rng, col).choose(rng).unwrap(), luma)
        })
        .collect::<Vec<_>>();

    GradientMap::with_map(map)
}

fn parse_quantize_hue(log: Log, rng: &mut impl Rng, effect: &Mapping) -> QuantizeHue {
    let param = effect.get("quantize-hue").unwrap();

    let hues = param
        .as_sequence()
        .expect("[quantize-hue] must be a list of hues/hue options.");

    let hues = hues
        .iter()
        .map(|hue| parse_f64_param(rng, hue) as f32)
        .collect::<Vec<_>>();

    let hues_str = hues
        .iter()
        .map(|h| format!("{h:.3}"))
        .collect::<Vec<String>>();

    log.apply_effect("quantize-hue", vec![("hues", format!("{:?}", hues_str))]);

    QuantizeHue::with_hues(hues)
}

fn parse_multiply_hue(log: Log, rng: &mut impl Rng, effect: &Mapping) -> MultiplyHue {
    let param = effect.get("multiply-hue").unwrap();

    let factor = parse_f64_param(rng, param) as f32;

    log.apply_effect("multiply-hue", vec![("factor", format!("{}", factor))]);

    MultiplyHue(factor)
}

fn parse_ordered(log: Log, rng: &mut impl Rng, effect: &Mapping) -> Ordered {
    let config = effect.get("ordered").unwrap();

    let strategy = &parse_property_as_str_param(rng, config, "strategy")
        .expect("[ordered] requires a [strategy] to be set.");

    let invert_chance = parse_property_as_f64_param(rng, config, "invert").unwrap_or(0.0);

    let palette = config
        .get("palette")
        .expect("[bayer] requires a [palette] to be set.");
    let palette = parse_palette(rng, palette);

    let mirror_chance = if let Some((mirror_chance, mirror_sets)) =
        config.get("mirror").map(|mirror| parse_mirror(rng, mirror))
    {
        Some((mirror_chance, mirror_sets))
    } else {
        None
    };

    let invert_chance = rng.gen_range(0.0..1.0);

    let (mut strategy, (param, mut parameters)) = match strategy.as_str() {
        "bayer" => {
            let size = parse_matrix_size(rng, config, strategy) as usize;

            (
                Bayer(size),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{}", size)),
                        ("invert", format!("{}", invert_chance)),
                    ],
                ),
            )
        }
        "diamonds" => {
            let size = parse_matrix_size(rng, config, strategy) as usize;
            (
                Diamonds(size),
                ((
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{}", size)),
                    ],
                )),
            )
        }
        "checkered-diamonds" => {
            let size = parse_matrix_size(rng, config, strategy) as usize;
            (
                CheckeredDiamonds(size),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{}", size)),
                    ],
                ),
            )
        }
        "stars" => (
            Stars,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "new-stars" => (
            NewStars,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "grid" => (
            Grid,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "trail" => (
            Trail,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "criss-cross" => (
            Crisscross,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "static" => (
            Static,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "wavy" => {
            let orientation = parse_orientation(rng, config, strategy);
            let o_str = match &orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            };
            (
                Wavy(orientation),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("orientation", format!("{}", o_str)),
                    ],
                ),
            )
        }
        "bootleg-bayer" => (
            BootlegBayer,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "diagonals" => (
            Diagonals,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "diagonals-big" => (
            DiagonalsBig,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "diamond-grid" => (
            DiamondGrid,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "speckle-squares" => (
            SpeckleSquares,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "scales" => (
            Scales,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "trail-scales" => (
            TrailScales,
            ("ordered.strategy", vec![("name", format!("{strategy}"))]),
        ),
        "diagonals-n" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;
            let direction = parse_diagonaldirection(rng, config, strategy);
            let increase = parse_increase_strategy(rng, config, strategy);

            (
                DiagonalsN {
                    n,
                    direction: direction.clone(),
                    increase: increase.clone(),
                },
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        (
                            "direction",
                            match direction {
                                DiagonalDirection::DownRight => "down-right".to_string(),
                                DiagonalDirection::UpRight => "up-right".to_string(),
                            },
                        ),
                        (
                            "increase",
                            match increase {
                                Increase::Linear(factor) => format!("linear ({factor})"),
                                Increase::Exponential(factor) => format!("exponential ({factor})"),
                            },
                        ),
                    ],
                ),
            )
        }
        "diagonal-tiles" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;

            (
                DiagonalTiles(n),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                    ],
                ),
            )
        }
        "bouncing-bowtie" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;

            (
                BouncingBowtie(n),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                    ],
                ),
            )
        }
        "scanline" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;
            let orientation = parse_orientation(rng, config, strategy);

            (
                ScanLine(n, orientation.clone()),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                        (
                            "orientation",
                            match orientation {
                                Orientation::Horizontal => "horizontal".to_string(),
                                Orientation::Vertical => "vertical".to_string(),
                            },
                        ),
                    ],
                ),
            )
        }
        "starburst" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;

            (
                Starburst(n),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                    ],
                ),
            )
        }
        "shiny-bowtie" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;

            (
                ShinyBowtie(n),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                    ],
                ),
            )
        }
        "marble-tile" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;

            (
                MarbleTile(n),
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                    ],
                ),
            )
        }
        "curve-path" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;
            let amplitude = parse_property_as_f64_param(rng, config, "amplitude").unwrap_or(1.0);
            let promotion = parse_property_as_f64_param(rng, config, "promotion").unwrap_or(0.0);
            let halt_threshold =
                parse_property_as_u64_param(rng, config, "halt-threshold").unwrap_or(100) as usize;

            (
                CurvePath {
                    n,
                    amplitude,
                    promotion,
                    halt_threshold,
                },
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                        ("amplitude", format!("{amplitude}")),
                        ("promotion", format!("{promotion}")),
                        ("halt-threshold", format!("{halt_threshold}")),
                    ],
                ),
            )
        }
        "zigzag" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;
            let halt_threshold =
                parse_property_as_u64_param(rng, config, "halt-threshold").unwrap_or(100) as usize;
            let wrapping = parse_wrapping_set(rng, config).choose(rng).unwrap().clone();

            let magnitude = parse_property_as_f64_tuple_param(rng, config, "magnitude", ("y", "x"));
            let promotion = parse_property_as_f64_tuple_param(rng, config, "promotion", ("y", "x"));

            let magnitude = (magnitude.0.unwrap_or(1.0), magnitude.1.unwrap_or(1.0));
            let promotion = (promotion.0.unwrap_or(0.0), promotion.1.unwrap_or(0.0));

            (
                ZigZag {
                    n,
                    halt_threshold,
                    wrapping: wrapping.clone(),
                    magnitude,
                    promotion,
                },
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                        ("magnitude", format!("{magnitude:?}")),
                        ("promotion", format!("{promotion:?}")),
                        (
                            "wrapping",
                            match wrapping {
                                Wrapping::None => "none".to_string(),
                                Wrapping::All => "all".to_string(),
                                Wrapping::Vertical => "vertical".to_string(),
                                Wrapping::Horizontal => "horizontal".to_string(),
                            },
                        ),
                        ("halt-threshold", format!("{halt_threshold}")),
                    ],
                ),
            )
        }
        "broken-spiral" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;
            let halt_threshold =
                parse_property_as_u64_param(rng, config, "halt-threshold").unwrap_or(100) as usize;

            let base_step = parse_property_as_f64_tuple_param(rng, config, "base-step", ("y", "x"));
            let base_step = (base_step.0.unwrap_or(1.0), base_step.1.unwrap_or(1.0));

            let oob_threshold = parse_property_as_u64_param(rng, config, "oob-threshold")
                .unwrap_or((n as f64 / (base_step.0.min(base_step.1))) as u64)
                as usize;
            let increment_by =
                parse_property_as_f64_param(rng, config, "increment-by").unwrap_or(0.0);
            let increment_in =
                parse_property_as_u64_param(rng, config, "increment-in").unwrap_or(1) as usize;

            let n = parse_matrix_size(rng, config, strategy) as usize;

            (
                BrokenSpiral {
                    n,
                    base_step,
                    oob_threshold,
                    increment_by,
                    increment_in,
                },
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{n}")),
                        ("base-step", format!("{base_step:?}")),
                        ("oob-threshold", format!("{oob_threshold}")),
                        ("increment-by", format!("{increment_by}")),
                        ("increment-in", format!("{increment_in}")),
                    ],
                ),
            )
        }
        "modulo-snake" => {
            let n = parse_matrix_size(rng, config, strategy) as usize;
            let increment_by =
                parse_property_as_f64_param(rng, config, "increment-by").unwrap_or(1.0);
            let modulo = parse_property_as_u64_param(rng, config, "modulo").unwrap_or(10) as usize;
            let iterations =
                parse_property_as_u64_param(rng, config, "iterations").unwrap_or(1) as usize;

            (
                ModuloSnake {
                    n,
                    increment_by,
                    modulo,
                    iterations,
                },
                (
                    "ordered.strategy",
                    vec![
                        ("name", format!("{strategy}")),
                        ("matrix-size", format!("{}", n)),
                        ("increment-by", format!("{increment_by}")),
                        ("modulo", format!("{modulo}")),
                        ("iterations", format!("{iterations}")),
                    ],
                ),
            )
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

    if mirror_chance.is_some() {
        let (chance, mirror_sets) = mirror_chance.unwrap();
        if rng.gen_range(0.0..1.0) <= chance {
            for mirrorline in mirror_sets {
                parameters.push((
                    "mirror",
                    match mirrorline {
                        MirrorLine::Upright(flip) => format!("upright [flip={}]", flip.0),
                        MirrorLine::Downright(flip) => format!("downright [flip={}]", flip.0),
                        MirrorLine::Horizontal(flip) => format!("horizontal [flip={}]", flip.0),
                        MirrorLine::Vertical(flip) => format!("vertical [flip={}]", flip.0),
                    },
                ));
                strategy = strategy.mirror(mirrorline);
            }
        };
    }
    strategy = if rng.gen_range(0.0..1.0) <= invert_chance {
        parameters.push(("invert", "true".to_string()));
        strategy.invert()
    } else {
        strategy
    };

    log.apply_effect("ordered.strategy", parameters);

    Ordered::new(palette, strategy)
}

fn parse_error_propagator<'a, 'b>(
    rng: &mut impl Rng,
    effect: &Mapping,
    algorithm_name: &str,
) -> ErrorPropagator<'a, 'b, WithPalette> {
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
        .expect(format!("[{algorithm_name}] requires a [palette] to be set.").as_str());

    let palette = parse_palette(rng, palette);

    propagator.with_palette(palette)
}

// parse common blocks

fn parse_palette(rng: &mut impl Rng, param: &serde_yaml::Value) -> Vec<Srgb> {
    if let Some(palette) = param.as_mapping() {
        let palette_type = palette
            .get("type")
            .expect("[palette] requires a [.type] to be specified.")
            .as_str()
            .expect("[palette.type] must be a string.");

        match palette_type {
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
                    .into_iter()
                    .map(|colour| colour.as_mapping().unwrap())
                    .map(|colour| parse_colour(rng, colour))
                    .collect::<Vec<_>>()
                    .concat()
            }
            "random_v2" => {
                let config = palette
                    .get("config")
                    .expect("if [palette.type] is \"random_v2\", [palette.config] must be present.")
                    .as_mapping()
                    .expect("[palette.config] must be a mapping with valid options.");

                generate_random_palette_v2(rng, config)
            }
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

fn generate_random_palette_v2(mut rng: &mut impl Rng, config: &Mapping) -> Vec<Srgb> {
    let max_lum = config
        .get("max_lum")
        .map_or(100.0, |param| parse_f64_param(rng, param));

    let min_lum = config
        .get("min_lum")
        .map_or(0.0, |param| parse_f64_param(rng, param));

    let (lum_strategy, lum_amnt) = parse_lum_strategy(rng, config);

    let inject = parse_inject(rng, config);

    let hue_strategies = parse_hue_strategies(rng, config);

    let chroma_strategy = parse_chroma_strategy(rng, config);

    let misc_flags = config.get("misc_flags").map(|param| {
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

    let mut flag_grayscale = false;
    let mut flag_lum_safeguard = false;
    let mut flag_extremes = false;
    let mut flag_single_lum = false;

    if let Some(flags) = misc_flags {
        if flags.contains(&"grayscale") {
            flag_grayscale = true
        }
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
                hues = vec![hues, generate_hue_neighbourhood(seed_hue, *size, *n, dist)].concat()
            }
            HueStrategy::Contrast { size, n, dist } => {
                hues = vec![
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
                hues = vec![
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
            LumStrategy::Random { unified } => {
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
        palette = vec![palette, lch_colours].concat();
    }

    palette
        .into_iter()
        .map(|color| color.into_color())
        .collect()
}
