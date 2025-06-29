use std::ops::Range;

use image_effects::prelude::IntoGradientLch;
use palette::{named, rgb::Rgb, IntoColor, Lch, Srgb};
use rand::{seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::util::{
        parse_property_as_f64_complex, parse_property_as_str, parse_property_as_u64_complex,
        parse_value_as_f64_sequence_complex,
    },
};

pub fn parse_palette(
    log: Log,
    rng: &mut impl Rng,
    param: &serde_yaml::Value,
) -> BaseResult<Vec<Srgb>> {
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
    let mut flag_grayscale = false;

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
        if flags.contains(&"grayscale") {
            flag_grayscale = true
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
            palette.iter_mut().for_each(|col| {
                if flag_grayscale {
                    col.chroma = 0.0;
                } else {
                    col.chroma = rng.gen_range(range.clone()) as f32;
                }
            });
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

#[allow(dead_code)]
pub enum LumStrategy {
    Exact(Vec<f64>),
    Random { unified: bool },
    Distributed,
    DistributedArea { overlap: Option<f64> },
    DistributedNudge { nudge_size: f64 },
}

// LUM STRATEGY
// ============
// This will determine how the luminescence values will be generated.
// Each HUE will be processed by the specified strategy into N variants,
//  where N is specified by the user.
// So, if there are H hues, and N lum-variants, there will be HxN total colours.
//
// Strategies here can have the following characteristics:
//  - Uniform: One set of luminescence values is generated, and applied for all hues.
//  - Chaotic: Every hue will have its own set of luminescence values.
//  - Seeded: The strategy activates once the *first* value is generated.
//
// META:
// range_size = how many variants to generate.
// strategy = which strategy to depend on.
//
// -- experimental --
// strategy_pool = a set of strategies to pick from. can specify odds.
//
// STRATEGIES:
// "exact":
//      variants will be generated for each specified LUM.
//      as a result, this will DISABLE min_lum and max_lum, AND range_size.
//      theoretically, since values are parsed to be *flexible*, you can
//          manually replicate some of the below strategies.
//      for example, "random" can be replicated by specifying four 0-100.0 ranges.
//
// "random":
//      the variants will be generated with *random luminescence*.
//      these levels will be random PER HUE.
//      this will result in an effect where *luminescence* contributes to color difference,
//
// "pseudo-random":
//      same as "random", except ALL HUES will share the same luminescence.
//      this ensures that ONLY HUEs will affect color difference.
//
// "distributed":
//      the variants will be generated to cover the entire span of LUM.
//      if N is 1, it will be the same as "random".
//      if N is 2, the variants will both be at the extremes.
//      for N > 2, additional variants should be split across the range.
//          N=?, #X will be at (X-2 / N-1)
//          N=3, #3 will be at 50% (3-2 / 3-1)
//          N=4, #3 will be at 33% (3-2 / 4-1)
//
// "distributed/areas":
//      same as distributed, except instead of static thresholds, it's within an area.
//      in this case, N specifies the *number of areas*
//      for example, N=3 results in 3 areas: 0.0~33.3, 33.3~66.6, 66.6~100.0.
//      a luminescence will then be generated within each area.
//      we can also choose to support an OVERLAP,
//          for example, OVERLAP=10.0 and N=3 would result in *these* areas:
//          0.0~43.3, 23.3~76.6, 56.6~100.0.
//
// "distributed/nudged":
//      same as "distributed", except each colour will have its luminescence *nudged* randomly.
//      this will have additional parameters:
//          "nudge_range": the luminescence will be "nudged" by an amount within this range.
//          "unified": if true, every colour will undergo the *same nudging*.
//              note that the per-luminescence nudge will still be random, but
//              the "nudge factors" will be shared per hue.
pub fn parse_lum_strategy(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<(LumStrategy, u64)> {
    let lum_strategy = value
        .get("lum-strategy")
        .expect("[lum-strategy] is required to be specified.");

    let strategy_type = parse_property_as_str(log, lum_strategy, "type")?
        .expect("[lum-strategy.type] must be string.");
    let count = parse_property_as_u64_complex(log, rng, lum_strategy, "count")?
        .expect("[lum-strategy.count] is required.");

    Ok((
        match strategy_type.as_str() {
            "exact" => LumStrategy::Exact(
                parse_value_as_f64_sequence_complex(log, rng, lum_strategy, "lums")?
                    .expect("[exact.lums] must be a list of floats."),
            ),
            "random" => {
                let unified = lum_strategy.get("unified").is_some_and(|param| {
                    param
                        .as_bool()
                        .expect("[random.unified] must be a boolean.")
                });

                LumStrategy::Random { unified }
            }
            "distributed" => LumStrategy::Distributed,
            "distributed/area" => {
                let overlap = parse_property_as_f64_complex(log, rng, lum_strategy, "overlap")?;
                LumStrategy::DistributedArea { overlap }
            }
            "distributed/nudge" => {
                if let Some(nudge_size) =
                    parse_property_as_f64_complex(log, rng, lum_strategy, "nudge-size")?
                {
                    LumStrategy::DistributedNudge { nudge_size }
                } else {
                    panic!("if [lum-strategy] was [distributed/nudge], [nudge-size] is required.")
                }
            }
            _ => panic!("{strategy_type} is not a valid lum_strategy."),
        },
        count,
    ))
}

pub enum HueDistribution {
    Linear,
    Random,
}
pub enum HueStrategy {
    Neighbour {
        size: f64,
        n: u64,
        dist: HueDistribution,
    },
    Contrast {
        size: f64,
        n: u64,
        dist: HueDistribution,
    },
    Penpal {
        size: f64,
        n: u64,
        dist: HueDistribution,
        distance: f64,
    },
    Cycle {
        n: u64,
    },
}

// HUE STRATEGY
// ============
// This determines how the hues will be generated.
// Luminescence will be handled by the LUM STRATEGY, which will then
//  create multiple instances of each hue generated here.
//
// Strategies here can have the following characteristics:
//  - Chaotic: Each hue is generated by some random algorithm, without dependencies.
//  - Seeded: Hues may be generated from a "seed".
//
// Strategies may be STACKED -- so this should be a list of strategies.
// Seeded strategies will utilise the *same seed*.
//
// For example, for a seed of 120, you can generate a plethora of hues by stacking:
// - neighbour (size=30, n=5, dist=linear)
// - neighbour (size=90, n=5, dist=random)
// - contrast (size=90, n=10)
// - cycle (n=3)
//
// STRATEGIES:
// "neighbour":
//      hues will be chosen within a "neighbourhood".
//      so for a "neighbourhood" size of 30, it will generate hues from -15 to +15
//      of the "seed" hue. they can be generated within the range, or distributed.
//      PARAMETERS: size, distribution
//
// "contrast":
//      hues will be chosen from the "opposite neighbourhood".
//      it's essentially the same as "neighbour", except when generating, the base
//          is 180-S instead of S.
//
// "penpal":
//      an extension on "contrast" -- it works the same, except now you control the distance.
//      it is effectively equal at distance=180 - but allows to pick any distance.
//      "contrast" remains due to being a common case, but may be removed for redundancy.
//
// "cycle":
//      hues will be generated linearly over a 360-degree span.
//      for example, N=1 will add a 180+S, N=2 will add 120+S and 240+S, etc...
pub fn parse_hue_strategies(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<Vec<HueStrategy>> {
    let raw_hue_strategies = value
        .get("hue-strategies")
        .expect("[hue-strategies] is required.")
        .as_sequence()
        .expect("[hue-strategies] must be a list of mappings.");

    let mut hue_strategies = vec![];

    for strategy in raw_hue_strategies {
        if !strategy.is_mapping() {
            panic!("[hue_strategies] entries must be mappings.")
        }

        let strategy_type = strategy
            .get("type")
            .expect("[hue-strategies] entries must have a [.type].")
            .as_str()
            .expect("[hue-strategies.#.type] must be a string.");

        let get_dist = |dist: &str| match dist {
            "linear" => HueDistribution::Linear,
            "random" => HueDistribution::Random,
            _ => panic!("{dist} is not a valid distribution."),
        };

        let iterations =
            parse_property_as_u64_complex(log, rng, strategy, "iterations")?.unwrap_or(1);

        let mut strategies = Vec::new();

        for _ in 0..iterations {
            strategies.push(match strategy_type {
                "neighbour" => {
                    let size = parse_property_as_f64_complex(log, rng, strategy, "size")?
                        .expect("[neighbour] strategy requires a [.size].");
                    let n = parse_property_as_u64_complex(log, rng, strategy, "count")?
                        .expect("[neighbour] strategy must specify a [.count]");
                    let dist = parse_property_as_str(log, strategy, "dist")?
                        .expect("[neighbour] strategy must be a string.");

                    HueStrategy::Neighbour {
                        size,
                        n,
                        dist: get_dist(&dist),
                    }
                }
                "contrast" => {
                    let size = parse_property_as_f64_complex(log, rng, strategy, "size")?
                        .expect("[contrast] strategy requires a [.size].");
                    let n = parse_property_as_u64_complex(log, rng, strategy, "count")?
                        .expect("[contrast] strategy must specify a [.count]");
                    let dist = parse_property_as_str(log, strategy, "dist")?
                        .expect("[contrast] strategy must be a string.");

                    HueStrategy::Contrast {
                        size,
                        n,
                        dist: get_dist(&dist),
                    }
                }
                "penpal" => {
                    let size = parse_property_as_f64_complex(log, rng, strategy, "size")?
                        .expect("[penpal] strategy requires a [.size].");
                    let n = parse_property_as_u64_complex(log, rng, strategy, "count")?
                        .expect("[penpal] strategy must specify a [.count]");
                    let dist = parse_property_as_str(log, strategy, "dist")?
                        .expect("[penpal] strategy must be a string.");
                    let distance = parse_property_as_f64_complex(log, rng, strategy, "distance")?
                        .expect("[penpal] strategy requires a [.distance].");

                    HueStrategy::Penpal {
                        size,
                        n,
                        dist: get_dist(&dist),
                        distance,
                    }
                }
                "cycle" => {
                    let n = parse_property_as_u64_complex(log, rng, value, "count")?
                        .expect("[cycle] strategy must specify a [.count]");

                    HueStrategy::Cycle { n }
                }
                _ => panic!("{strategy_type} is not a valid hue_strategy."),
            });
        }

        hue_strategies.push(strategies);
    }

    Ok(hue_strategies.into_iter().flatten().collect())
}

pub enum ChromaStrategy {
    Random(Range<f64>),
}

pub fn parse_chroma_strategy(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<ChromaStrategy> {
    let chroma_strategy = value
        .get("chroma-strategy")
        .expect("[chroma_strategy] is required.")
        .as_mapping()
        .expect("[chroma-strategy] must be a mapping.");

    let strategy_name = chroma_strategy
        .get("type")
        .expect("[chroma-strategy.type] must be present.")
        .as_str()
        .expect("[chroma-strategy.type] must be a string.");

    match strategy_name {
        "random" => {
            let range_start =
                parse_property_as_f64_complex(log, rng, value, "range-start")?.unwrap_or(0.0);
            let range_end =
                parse_property_as_f64_complex(log, rng, value, "range-end")?.unwrap_or(128.0);

            Ok(ChromaStrategy::Random(range_start..range_end))
        }
        _ => panic!("{strategy_name} is not a valid chroma_strategy."),
    }
}

// figure out how the hell this works.
pub fn parse_inject(log: Log, rng: &mut impl Rng, config: &Value) -> Option<Vec<Rgb>> {
    log.alert("okay so. inject doesn't work too well. take a look at its code when you can.");
    config.get("inject").map(|param| {
        let inject = param
            .as_mapping()
            .expect("[palette.config.inject] must be a mapping.");

        inject
            .get("colours")
            .map(|param| {
                let colours = param
                    .as_sequence()
                    .expect("[palette.colours] must be a list of valid colours");

                colours
                    .iter()
                    .map(|colour| colour.as_mapping().unwrap())
                    .map(|colour| parse_colour(log, rng, &Value::Mapping(colour.clone())).unwrap())
                    .collect::<Vec<_>>()
                    .concat()
            })
            .expect("if [palette.config.inject] is specified, it must have a [.colours] property.")
    })
}

pub fn parse_colour(log: Log, rng: &mut impl Rng, param: &Value) -> BaseResult<Vec<Srgb>> {
    Ok(if let Some(rgb) = param.get("rgb") {
        let colour = parse_rgb(rgb);
        let gradient = if let Some(shades) = param.get("shades") {
            let shades = shades
                .as_u64()
                .expect("[palette.shades] must be a positive integer.");
            colour.build_gradient_lch(shades as u16)
        } else {
            vec![colour]
        };
        gradient
    } else if let Some(amnt) = parse_property_as_u64_complex(log, rng, param, "random")? {
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
        panic!("wtf kind colour did you come up with here????")
    })
}

pub fn parse_rgb(value: &serde_yaml::Value) -> Srgb {
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
            panic!(
                "There should only be 3 RGB components. Found {}.",
                components.len()
            );
        }

        if components.iter().all(|f| f.is_f64()) {
            let components = components
                .iter()
                .map(|c| c.as_f64().unwrap())
                .collect::<Vec<_>>();
            Srgb::new(
                components[0] as f32,
                components[1] as f32,
                components[2] as f32,
            )
        } else if components.iter().all(|f| f.is_u64()) {
            let components = components
                .iter()
                .map(|c| c.as_u64().unwrap() as u8)
                .collect::<Vec<_>>();
            Srgb::<u8>::new(components[0], components[1], components[2]).into_format()
        } else {
            panic!("uh oh components are bad");
        }
    } else {
        panic!("uh oh rgb is bad");
    }
}

pub fn gen_with_random_lightness(rng: &mut impl Rng, min: f32, max: f32) -> Lch {
    Lch::new(
        rng.gen_range(min..=max),
        rng.gen_range(0.0..128.0),
        rng.gen_range(0.0..360.0),
    )
}

pub fn gen_with_lightness(rng: &mut impl Rng, lum: f32) -> Lch {
    Lch::new(lum, rng.gen_range(0.0..128.0), rng.gen_range(0.0..360.0))
}

pub fn generate_random_palette(mut rng: &mut impl Rng) -> Vec<Srgb> {
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
