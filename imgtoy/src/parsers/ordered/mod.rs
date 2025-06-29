use image_effects::dither::ordered::{Ordered, OrderedStrategy};
use palette::named;
use rand::{seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::{
        modifiers::{
            checker::parse_checker,
            diagonal_direction::parse_diagonaldirection,
            increase_strategy::parse_increase_strategy,
            mirror::parse_mirror,
            orientation::parse_orientation,
            rotation::parse_rotation,
            simple::{parse_blur, parse_exponentiate},
            wrapping::parse_wrapping_set,
        },
        palette::parse_palette,
        properties::parse_matrix_size,
        util::{
            parse_property_as_f64_complex, parse_property_as_f64_tuple_param,
            parse_property_as_str, parse_property_as_u64_complex,
        },
    },
};

pub enum OrderedKind {
    Bayer,
    Diamonds,
    CheckeredDiamonds,
    Stars,
    NewStars,
    Grid,
    Trail,
    CrissCross,
    Static,
    Wavy,
    BootlegBayer,
    Diagonals,
    DiagonalsBig,
    DiamondGrid,
    SpeckleSquares,
    Scales,
    TrailScales,
    DiagonalsN,
    DiagonalTiles,
    BouncingBowtie,
    Scanline,
    Starburst,
    ShinyBowtie,
    MarbleTile,
    CurvePath,
    Zigzag,
    BrokenSpiral,
    ModuloSnake,
}

impl From<&str> for OrderedKind {
    fn from(value: &str) -> Self {
        match value {
            "bayer" => Self::Bayer,
            "diamonds" => Self::Diamonds,
            "checkered-diamonds" => Self::CheckeredDiamonds,
            "stars" => Self::Stars,
            "new-stars" => Self::NewStars,
            "grid" => Self::Grid,
            "trail" => Self::Trail,
            "criss-cross" => Self::CrissCross,
            "static" => Self::Static,
            "wavy" => Self::Wavy,
            "bootleg-bayer" => Self::BootlegBayer,
            "diagonals" => Self::Diagonals,
            "diagonals-big" => Self::DiagonalsBig,
            "diamond-grid" => Self::DiamondGrid,
            "speckle-squares" => Self::SpeckleSquares,
            "scales" => Self::Scales,
            "trail-scales" => Self::TrailScales,
            "diagonals-n" => Self::DiagonalsN,
            "diagonal-tiles" => Self::DiagonalTiles,
            "bouncing-bowtie" => Self::BouncingBowtie,
            "scanline" => Self::Scanline,
            "starburst" => Self::Starburst,
            "shiny-bowtie" => Self::ShinyBowtie,
            "marble-tile" => Self::MarbleTile,
            "curve-path" => Self::CurvePath,
            "zigzag" => Self::Zigzag,
            "broken-spiral" => Self::BrokenSpiral,
            "modulo-snake" => Self::ModuloSnake,
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
                panic!("{value} is an invalid [ordered.strategy]. Allowed strategies are: {strategies:?}");
            }
        }
    }
}

pub fn parse_ordered(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Ordered> {
    let value = value.get("ordered").unwrap();

    let invert_chance = parse_property_as_f64_complex(log, rng, value, "invert")?.unwrap_or(0.0);

    let palette = value
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

    let mirror = value.get("mirror").map(|mirror| {
        log.begin_category("mirror").expect("fucked up when beginning the mirror category for some reason");
        let mirror = parse_mirror(log, rng, mirror).expect("okay yeah if you see this, something fucked up in the mirrors. good luck. just look for the code [X93HC] and you'll find this in the source code.");
        log.end_category().expect("fucked up when ending the category for some reason");
        mirror
    });

    let blur = parse_blur(log, rng, value)?;
    let exponentiate = parse_exponentiate(log, rng, value)?;
    let rotate = parse_rotation(log, rng, value)?;
    let checker = parse_checker(log, rng, value)?;

    log.begin_category("strategies")?;

    let strategies = parse_strategies(log, rng, value)?;

    let mut strategy = strategies.choose(rng).unwrap().clone();

    log.end_category()?;

    if let Some((chance, mirror_sets)) = mirror {
        if rng.gen_range(0.0..1.0) <= chance {
            for mirrorline in mirror_sets {
                strategy = strategy.mirror(mirrorline);
            }
        };
    }

    strategy = if let Some(blur) = blur {
        strategy.blur(blur as usize)
    } else {
        strategy
    };

    strategy = if let Some(exponentiate) = exponentiate {
        strategy.exponentiate(exponentiate)
    } else {
        strategy
    };

    strategy = if let Some(rotate) = rotate {
        strategy.rotate(rotate)
    } else {
        strategy
    };

    strategy = if let Some(checker) = checker {
        strategy.checker(checker)
    } else {
        strategy
    };

    strategy = if rng.gen_range(0.0..1.0) <= invert_chance {
        strategy.invert()
    } else {
        strategy
    };

    Ok(Ordered::new(palette, strategy))
}

pub fn parse_ordered_kind(log: Log, value: &Value) -> BaseResult<OrderedKind> {
    Ok(value
        .as_mapping()
        .expect("oopsies, should have an ordered kind buddy")
        .keys()
        .next()
        .unwrap()
        .as_str()
        .expect("strategy must have an ordered type as its name")
        .into())
}

pub fn parse_strategies(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<Vec<OrderedStrategy>> {
    let strategies = value
        .get("strategies")
        .expect("[strategies] was not present - is required.")
        .as_sequence()
        .expect("[strategies] must be a list - wasn't.");

    Ok(strategies
        .iter()
        .enumerate()
        .map(|(i, strategy)| {
            strategy
                .as_mapping()
                .expect(format!("[strategies.{i}] must be a map - wasn't.").as_str())
        })
        .map(|strategy| {
            let keys = strategy.keys().len();
            if keys != 1 {
                panic!("only one key [the strategy name] is accepted by ordered - found {keys}");
            }
            parse_strategy(
                log,
                rng,
                &Value::Mapping(strategy.clone()),
                strategy.keys().next().unwrap().as_str().expect("what????"),
            )
            .expect("failure when parsing strategy. please check log.")
        })
        .collect::<Vec<_>>())
}

pub fn parse_strategy(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    name: &str,
) -> BaseResult<OrderedStrategy> {
    let kind = parse_ordered_kind(log, value)?;

    let value: &Value = value.get(name).unwrap();

    log.begin_category(name);

    let mut strategy = match kind {
        OrderedKind::Bayer => {
            let size = parse_matrix_size(log, rng, value)? as usize;

            OrderedStrategy::Bayer(size)
        }
        OrderedKind::Diamonds => {
            let size = parse_matrix_size(log, rng, value)? as usize;
            OrderedStrategy::Diamonds(size)
        }
        OrderedKind::CheckeredDiamonds => {
            let size = parse_matrix_size(log, rng, value)? as usize;
            OrderedStrategy::CheckeredDiamonds(size)
        }
        OrderedKind::Stars => OrderedStrategy::Stars,
        OrderedKind::NewStars => OrderedStrategy::NewStars,
        OrderedKind::Grid => OrderedStrategy::Grid,
        OrderedKind::Trail => OrderedStrategy::Trail,
        OrderedKind::CrissCross => OrderedStrategy::Crisscross,
        OrderedKind::Static => OrderedStrategy::Static,
        OrderedKind::Wavy => {
            let orientation = parse_orientation(log, rng, value)?;
            OrderedStrategy::Wavy(orientation)
        }
        OrderedKind::BootlegBayer => OrderedStrategy::BootlegBayer,
        OrderedKind::Diagonals => OrderedStrategy::Diagonals,
        OrderedKind::DiagonalsBig => OrderedStrategy::DiagonalsBig,
        OrderedKind::DiamondGrid => OrderedStrategy::DiamondGrid,
        OrderedKind::SpeckleSquares => OrderedStrategy::SpeckleSquares,
        OrderedKind::Scales => OrderedStrategy::Scales,
        OrderedKind::TrailScales => OrderedStrategy::TrailScales,
        OrderedKind::DiagonalsN => {
            let n = parse_matrix_size(log, rng, value)? as usize;
            let direction = parse_diagonaldirection(log, rng, value)?;
            let increase = parse_increase_strategy(log, rng, value)?;

            OrderedStrategy::DiagonalsN {
                n,
                direction: direction.clone(),
                increase: increase.clone(),
            }
        }
        OrderedKind::DiagonalTiles => {
            let n = parse_matrix_size(log, rng, value)? as usize;

            OrderedStrategy::DiagonalTiles(n)
        }
        OrderedKind::BouncingBowtie => {
            let n = parse_matrix_size(log, rng, value)? as usize;

            OrderedStrategy::BouncingBowtie(n)
        }
        OrderedKind::Scanline => {
            let n = parse_matrix_size(log, rng, value)? as usize;
            let orientation = parse_orientation(log, rng, value);

            OrderedStrategy::ScanLine(n, orientation?.clone())
        }
        OrderedKind::Starburst => {
            let n = parse_matrix_size(log, rng, value)? as usize;

            OrderedStrategy::Starburst(n)
        }
        OrderedKind::ShinyBowtie => {
            let n = parse_matrix_size(log, rng, value)? as usize;

            OrderedStrategy::ShinyBowtie(n)
        }
        OrderedKind::MarbleTile => {
            let n = parse_matrix_size(log, rng, value)? as usize;

            OrderedStrategy::MarbleTile(n)
        }
        OrderedKind::CurvePath => {
            let n = parse_matrix_size(log, rng, value)? as usize;
            let amplitude =
                parse_property_as_f64_complex(log, rng, value, "amplitude")?.unwrap_or(1.0);
            let promotion =
                parse_property_as_f64_complex(log, rng, value, "promotion")?.unwrap_or(0.0);
            let halt_threshold = parse_property_as_u64_complex(log, rng, value, "halt-threshold")?
                .unwrap_or(100) as usize;

            OrderedStrategy::CurvePath {
                n,
                amplitude,
                promotion,
                halt_threshold,
            }
        }
        OrderedKind::Zigzag => {
            let n = parse_matrix_size(log, rng, value)? as usize;
            let halt_threshold = parse_property_as_u64_complex(log, rng, value, "halt-threshold")?
                .unwrap_or(100) as usize;
            let wrapping = parse_wrapping_set(log, rng, value)?
                .choose(rng)
                .unwrap()
                .clone();

            let magnitude =
                parse_property_as_f64_tuple_param(log, rng, value, "magnitude", ("y", "x"))?;
            let promotion =
                parse_property_as_f64_tuple_param(log, rng, value, "promotion", ("y", "x"))?;

            let magnitude = (magnitude.0.unwrap_or(1.0), magnitude.1.unwrap_or(1.0));
            let promotion = (promotion.0.unwrap_or(0.0), promotion.1.unwrap_or(0.0));

            OrderedStrategy::ZigZag {
                n,
                halt_threshold,
                wrapping: wrapping.clone(),
                magnitude,
                promotion,
            }
        }
        OrderedKind::BrokenSpiral => {
            let n = parse_matrix_size(log, rng, value)? as usize;

            let base_step =
                parse_property_as_f64_tuple_param(log, rng, value, "base-step", ("y", "x"))?;
            let base_step = (base_step.0.unwrap_or(1.0), base_step.1.unwrap_or(1.0));

            let oob_threshold = parse_property_as_u64_complex(log, rng, value, "oob-threshold")?
                .unwrap_or((n as f64 / (base_step.0.min(base_step.1))) as u64)
                as usize;
            let increment_by =
                parse_property_as_f64_complex(log, rng, value, "increment-by")?.unwrap_or(0.0);
            let increment_in = parse_property_as_u64_complex(log, rng, value, "increment-in")?
                .unwrap_or(1) as usize;

            let n = parse_matrix_size(log, rng, value)? as usize;

            OrderedStrategy::BrokenSpiral {
                n,
                base_step,
                oob_threshold,
                increment_by,
                increment_in,
            }
        }
        OrderedKind::ModuloSnake => {
            let n = parse_matrix_size(log, rng, value)? as usize;
            let increment_by =
                parse_property_as_f64_complex(log, rng, value, "increment-by")?.unwrap_or(1.0);
            let modulo =
                parse_property_as_u64_complex(log, rng, value, "modulo")?.unwrap_or(10) as usize;
            let iterations =
                parse_property_as_u64_complex(log, rng, value, "iterations")?.unwrap_or(1) as usize;

            OrderedStrategy::ModuloSnake {
                n,
                increment_by,
                modulo,
                iterations,
            }
        }
    };

    log.end_category()?;

    Ok(strategy)
}
