use std::ops::Range;

use image_effects::prelude::IntoGradientLch;
use palette::{rgb::Rgb, Srgb};
use rand::Rng;
use serde_yaml::Mapping;

use super::{parse_f64_param, parse_u64_param};

pub enum LumStrategy {
    Exact(Vec<f64>),
    Random { unified: bool },
    Distributed,
    DistributedArea { overlap: Option<f64> },
    DistributedNudge { nudge_size: f64 }
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
pub fn parse_lum_strategy(rng: &mut impl Rng, config: &Mapping) -> (LumStrategy, u64) {
    let lum_strategy = config
        .get("lum_strategy").expect("[lum_strategy] is required.")
        .as_mapping().expect("[lum_strategy] must be a mapping.");

    let strategy_type = lum_strategy
        .get("type").expect("[lum_strategy] must have a [.type]")
        .as_str().expect("[lum_strategy.type] must be string.");

    let lum_amnt = parse_u64_param(rng, lum_strategy.get("count").expect("[lum_strategy] must have a [.count]"));

    (match strategy_type {
        "exact" => {
            let lums = lum_strategy
                .get("lums").expect("[exact] lum_strategy must have [lums] specified.")
                .as_sequence().expect("[exact.lums] must be a list of floats.");

            LumStrategy::Exact(lums.iter().map(|param| parse_f64_param(rng, param)).collect())
        },
        "random" => {
            let unified = lum_strategy
                .get("unified")
                .is_some_and(|param| param.as_bool().expect("[random.unified] must be a boolean."));

            LumStrategy::Random { unified }
        },
        "distributed" => {
            LumStrategy::Distributed
        },
        "distributed/area" => {
            let overlap = lum_strategy
                .get("overlap")
                .map(|param| parse_f64_param(rng, param));

            LumStrategy::DistributedArea { overlap }
        },
        "distributed/nudge" => {
            let nudge_size = lum_strategy
                .get("nudge_size")
                .map_or_else(|| panic!("[distributed/nudge] must have [.nudge_size] specified."), |param| parse_f64_param(rng, param));

            LumStrategy::DistributedNudge { nudge_size }
        },
        _ => panic!("{strategy_type} is not a valid lum_strategy."),
    }, lum_amnt)
}

pub enum HueDistribution { Linear, Random }
pub enum HueStrategy {
    Neighbour { size: f64, n: u64, dist: HueDistribution },
    Contrast { size: f64, n: u64, dist: HueDistribution },
    Penpal { size: f64, n: u64, dist: HueDistribution, distance: f64 },
    Cycle { n: u64 }
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
pub fn parse_hue_strategies(rng: &mut impl Rng, config: &Mapping) -> Vec<HueStrategy> {
    let hue_strategies = config
        .get("hue_strategies").expect("[hue_strategies] is required.")
        .as_sequence().expect("[hue_strategies] must be a list of mappings.");

    hue_strategies.iter().flat_map(|strategy| {
        if !strategy.is_mapping() {
            panic!("[hue_strategies] entries must be mappings.")
        }

        let strategy_type = strategy
            .get("type").expect("[hue_strategies] entries must have a [.type].")
            .as_str().expect("[hue_strategies.#.type] must be a string.");

        let get_dist = |dist: &str| match dist {
            "linear" => HueDistribution::Linear,
            "random" => HueDistribution::Random,
            _ => panic!("{dist} is not a valid distribution."),
        };

        let iterations = strategy.get("iterations").map(|param| parse_u64_param(rng, param)).unwrap_or(1);
        let mut strategies = Vec::new();

        for i in 0..iterations {
            strategies.push(match strategy_type {
                "neighbour" => {
                    let size = parse_f64_param(rng, strategy.get("size").expect("[neighbour] strategy must specify a [.size]."));
                    let n = parse_u64_param(rng, strategy.get("count").expect("[neighbour] strategy must specify a [.count]"));
                    let dist = strategy
                        .get("dist").expect("[neighbour] strategy must specify a [.dist]")
                        .as_str().expect("[neighbour.dist] must be a string.");

                    HueStrategy::Neighbour { size, n, dist: get_dist(dist) }
                },
                "contrast" => {
                    let size = parse_f64_param(rng, strategy.get("size").expect("[contrast] strategy must specify a [.size]."));
                    let n = parse_u64_param(rng, strategy.get("count").expect("[contrast] strategy must specify a [.count]"));
                    let dist = strategy
                        .get("dist").expect("[contrast] strategy must specify a [.dist]")
                        .as_str().expect("[contrast.dist] must be a string.");

                    HueStrategy::Contrast { size, n, dist: get_dist(dist) }
                },
                "penpal" => {
                    let size = parse_f64_param(rng, strategy.get("size").expect("[penpal] strategy must specify a [.size]."));
                    let n = parse_u64_param(rng, strategy.get("count").expect("[penpal] strategy must specify a [.count]"));
                    let dist = strategy
                        .get("dist").expect("[penpal] strategy must specify a [.dist]")
                        .as_str().expect("[penpal.dist] must be a string.");
                    let distance = parse_f64_param(rng, strategy.get("distance").expect("[penpal] strategy must specify a [.distance]"));

                    HueStrategy::Penpal { size, n, dist: get_dist(dist), distance }
                },
                "cycle" => {
                    let n = parse_u64_param(rng, strategy.get("count").expect("[cycle] strategy must specify a [.count]"));

                    HueStrategy::Cycle { n }
                },
                _ => panic!("{strategy_type} is not a valid hue_strategy."),
            });
        }

        strategies
    }).collect()
}

pub enum ChromaStrategy {
    Random(Range<f64>)
}

pub fn parse_chroma_strategy(rng: &mut impl Rng, config: &Mapping) -> ChromaStrategy {
    let chroma_strategy = config
        .get("chroma_strategy").expect("[chroma_strategy] is required.")
        .as_mapping().expect("[chroma_strategy] must be a mapping.");

    let strategy_name = chroma_strategy
        .get("type").expect("[chroma_strategy.type] must be present.")
        .as_str().expect("[chroma_strategy.type] must be a string.");

    match strategy_name {
        "random" => {
            let range_start = chroma_strategy.get("range_start").map(|param| parse_f64_param(rng, param)).unwrap_or(0.0);
            let range_end = chroma_strategy.get("range_end").map(|param| parse_f64_param(rng, param)).unwrap_or(128.0);

            ChromaStrategy::Random(range_start..range_end)
        },
        _ => panic!("{strategy_name} is not a valid chroma_strategy."),
    }
}

pub fn parse_inject(rng: &mut impl Rng, config: &Mapping) -> Option<Vec<Rgb>> {
    config.get("inject").map(|param| {
        let inject = param.as_mapping().expect("[palette.config.inject] must be a mapping.");

        inject.get("colours").map(|param| {
            let colours = param.as_sequence().expect("[palette.colours] must be a list of valid colours");

            colours.iter()
                .map(|colour| colour.as_mapping().unwrap())
                .map(|colour| parse_colour(rng, colour))
                .collect::<Vec<_>>()
                .concat()
        }).expect("if [palette.config.inject] is specified, it must have a [.colours] property.")
    })
}

pub fn parse_colour(rng: &mut impl Rng, param: &Mapping) -> Vec<Srgb> {    
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