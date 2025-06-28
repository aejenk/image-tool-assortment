use image_effects::dither::ordered::{
    algorithms::Wrapping, DiagonalDirection, Flip, Increase, MirrorLine, Orientation,
};
use rand::{seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::util::{
        parse_property_as_f64, parse_property_as_f64_complex, parse_property_as_u64_complex,
    },
};

/// The option passed should be a *mapping* that represents a mirror option.
/// Such an option has multiple properties:
/// - chance: a float from 0.0 to 1.0 that represents the chance of application
/// - directions: a list of sets of directions to apply. one should be chosen at random.
///     - downright
///     - upright
///     - horizontal
///     - vertical
///
/// This will return a (f64, Vec<MirrorLine>) that represents the chance
/// alongside the Mirrorline.
pub fn parse_mirror(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<(f64, Vec<MirrorLine>)> {
    let chance = parse_chance(log, value)?;

    log.begin_category("mirror-set")?;
    let mirror_set = parse_mirror_direction_set(log, rng, value)?;
    log.end_category()?;

    Ok((chance, mirror_set))
}

pub fn parse_chance(log: Log, value: &Value) -> BaseResult<f64> {
    Ok(parse_property_as_f64(log, value, "chance", Some(0.0))?
        .expect("[mirror.chance] must be a float."))
}

pub fn parse_mirror_direction_set(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<Vec<MirrorLine>> {
    let directions = value
        .get("directions")
        .expect("[mirror.directions] must specify at least one direction.")
        .as_sequence()
        .expect("[mirror.directions] must be a list.");

    let flip_chance = parse_property_as_f64_complex(log, rng, value, "flip")?.unwrap_or(0.0);

    let directions = directions
        .iter()
        .map(|direction_set| {
            let direction_set = direction_set
                .as_sequence()
                .expect("[mirror.directions[$]] should be a sequence of strings.");
            direction_set
                .iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .expect("[mirror.directions[$][$]] must be a string.")
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let directions = directions.choose(rng).unwrap();

    let mut get_flip = || Flip(rng.gen_range(0.0..1.0) < flip_chance);

    let mut mirror_set = vec![];

    for (i, direction_name) in directions.iter().enumerate() {
        let direction = match *direction_name {
            "downright" => MirrorLine::Downright(get_flip()),
            "upright" => MirrorLine::Upright(get_flip()),
            "horizontal" => MirrorLine::Downright(get_flip()),
            "vertical" => MirrorLine::Downright(get_flip()),
            _ => {
                panic!(
                    "[mirror.directions[{i}].direction] found invalid direction [{direction_name}]"
                )
            }
        };

        log.state_property(
            format!("#{i:03}"),
            match direction {
                MirrorLine::Downright(Flip(flip)) => format!("downright, flip={flip}"),
                MirrorLine::Upright(Flip(flip)) => format!("upright, flip={flip}"),
                MirrorLine::Horizontal(Flip(flip)) => format!("horizontal, flip={flip}"),
                MirrorLine::Vertical(Flip(flip)) => format!("vertical, flip={flip}"),
            },
        )?;

        mirror_set.push(direction);
    }

    Ok(mirror_set)
}

pub fn parse_wrapping_set(log: Log, _: &mut impl Rng, value: &Value) -> BaseResult<Vec<Wrapping>> {
    let wrappings = value
        .get("wrappings")
        .expect("[wrappings] must specify at least one direction.")
        .as_sequence()
        .expect("[wrappings] must be a list.");

    let wrappings = wrappings
        .iter()
        .map(|wrapping| wrapping.as_str().expect("[wrappings[$]] must be a string."))
        .collect::<Vec<_>>();

    log.state_property("wrappings", format!("{wrappings:?}"))?;

    let mut parsed_wrappings = vec![];

    for wrapping in wrappings {
        parsed_wrappings.push(match wrapping {
            "horizontal" => Wrapping::Horizontal,
            "vertical" => Wrapping::Vertical,
            "all" => Wrapping::All,
            "none" => Wrapping::None,
            _ => panic!("[wrappings[$]] found invalid wrapping [{wrapping}]"),
        });
    }

    Ok(parsed_wrappings)
}

pub fn parse_increase_strategy(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    strategy: &str,
) -> BaseResult<Increase> {
    let increase_strategy = value.get("increase-strategy").unwrap_or_else(|| {
        panic!("[ordered.strategy ({strategy})] requires [ordered.increase-strategy]")
    });

    let strategy_type = increase_strategy
        .get("type")
        .expect("[ordered.strategy.increase-strategy] must have a [type]");

    let factor = parse_property_as_u64_complex(log, rng, increase_strategy, "factor")?
        .expect("Expected [ordered.strategy.increase-strategy] to have a u64.");

    log.state_property("factor", factor)?;

    let increase = match strategy_type {
        Value::Mapping(mapping) => {
            let linear = mapping.get("linear").map(|ratio| ratio.as_f64().unwrap_or_else(|| { panic!("{}", "[ordered.orientation.horizontal] must be a float.".to_string()) }));
            let exponential = mapping.get("exponential").map(|ratio| ratio.as_f64().unwrap_or_else(|| { panic!("{}", "[ordered.orientation.vertical] must be a float.".to_string()) }));

            match (linear, exponential) {
                (Some(l), Some(e)) => {
                    let target = rng.gen_range(0.0..l+e);
                    if target < l {
                        Increase::Linear(factor as u8)
                    } else {
                        Increase::Exponential(factor as u8)
                    }
                }
                (None, _) | (_, None) => panic!("[ordered.increase-strategy] must have a [down-right] and [up-right] float.")
            }
        },
        Value::String(strategy_type) => {
            match strategy_type.as_str() {
                "linear" => Increase::Linear(factor as u8),
                "exponential" => Increase::Exponential(factor as u8),
                _ => panic!("[ordered.increase-strategy.type] must be 'linear' or 'exponential"),
            }
        },
        _ => panic!("[ordered.orientation] must be a mapping of ratios, or one of 'down-right' / 'up-right'")
    };

    match increase {
        Increase::Linear(f) => log.state_property("linear-factor", f)?,
        Increase::Exponential(f) => log.state_property("exponential-factor", f)?,
    };

    Ok(increase)
}

pub fn parse_matrix_size(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    strategy: &str,
) -> BaseResult<u64> {
    Ok(
        parse_property_as_u64_complex(log, rng, value, "matrix-size")?
            .expect(format!("[ordered.{strategy}] must have a u64 param").as_str()),
    )
}

pub fn parse_orientation(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    strategy: &str,
) -> BaseResult<Orientation> {
    let orientation = value.get("orientation").unwrap_or_else(|| {
        panic!("[ordered.strategy ({strategy})] requires [ordered.orientation]")
    });

    let orientation = match orientation {
        Value::Mapping(mapping) => {
            let horizontal_ratio = mapping.get("horizontal").map(|ratio| ratio.as_f64().unwrap_or_else(|| { panic!("{}", "[ordered.orientation.horizontal] must be a float.".to_string()) }));
            let vertical_ratio = mapping.get("vertical").map(|ratio| ratio.as_f64().unwrap_or_else(|| { panic!("{}", "[ordered.orientation.vertical] must be a float.".to_string()) }));

            match (horizontal_ratio, vertical_ratio) {
                (Some(hr), Some(vr)) => {
                    let target = rng.gen_range(0.0..hr+vr);
                    if target < hr {
                        Orientation::Horizontal
                    } else {
                        Orientation::Vertical
                    }
                }
                (None, _) | (_, None) => panic!("[ordered.orientation] must have a [horizontal] and [vertical] float.")
            }
        },
        Value::String(orientation) => {
            match orientation.as_str() {
                "horizontal" => Orientation::Horizontal,
                "vertical" => Orientation::Vertical,
                _ => panic!("[ordered.orientation] must be 'horizontal', 'vertical', or a mapping of ratios.")
            }
        },
        _ => panic!("[ordered.orientation] must be a mapping of ratios, or one of 'vertical' / 'horizontal'")
    };

    match orientation {
        Orientation::Horizontal => log.state_property("orientation", "horizontal")?,
        Orientation::Vertical => log.state_property("orientation", "vertical")?,
    };

    Ok(orientation)
}

pub fn parse_diagonaldirection(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
    strategy: &str,
) -> BaseResult<DiagonalDirection> {
    let diagonaldirection = value.get("diagonal-direction").unwrap_or_else(|| {
        panic!("[ordered.strategy ({strategy})] requires [ordered.diagonal-direction]")
    });

    let diagonal_direction = match diagonaldirection {
        Value::Mapping(mapping) => {
            let dr_ratio = mapping.get("down-right").map(|ratio| ratio.as_f64().unwrap_or_else(|| { panic!("{}", "[ordered.orientation.horizontal] must be a float.".to_string()) }));
            let ur_ratio = mapping.get("up-right").map(|ratio| ratio.as_f64().unwrap_or_else(|| { panic!("{}", "[ordered.orientation.vertical] must be a float.".to_string()) }));

            match (dr_ratio, ur_ratio) {
                (Some(dr), Some(ur)) => {
                    let target = rng.gen_range(0.0..dr+ur);
                    if target < dr {
                        DiagonalDirection::DownRight
                    } else {
                        DiagonalDirection::UpRight
                    }
                }
                (None, _) | (_, None) => panic!("[ordered.orientation] must have a [down-right] and [up-right] float.")
            }
        },
        Value::String(orientation) => {
            match orientation.as_str() {
                "down-right" => DiagonalDirection::DownRight,
                "up-right" => DiagonalDirection::UpRight,
                _ => panic!("[ordered.orientation] must be 'down-right', 'up-right', or a mapping of ratios.")
            }
        },
        _ => panic!("[ordered.orientation] must be a mapping of ratios, or one of 'down-right' / 'up-right'")
    };

    match diagonal_direction {
        DiagonalDirection::DownRight => log.state_property("diagonal-direction", "down-right")?,
        DiagonalDirection::UpRight => log.state_property("diagonal-direction", "up-right")?,
    };

    Ok(diagonal_direction)
}
// end
