use image_effects::dither::ordered::{DiagonalDirection, Increase, MirrorLine, Orientation};
use rand::{seq::SliceRandom, Rng};
use serde_yaml::Value;

use crate::parsers::parse_u64_param;

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
pub fn parse_mirror(rng: &mut impl Rng, value: &Value) -> (f64, Vec<MirrorLine>) {
    let chance = parse_chance(rng, value);
    let mirror_sets = parse_mirror_direction_set(rng, value);

    (chance, mirror_sets.choose(rng).unwrap().clone())
}

pub fn parse_chance(rng: &mut impl Rng, value: &Value) -> f64 {
    value.get("chance")
        .map(|val| val.as_f64())
        .unwrap_or(Some(0.0))
        .expect("[mirror.chance] must be a float.")
}

pub fn parse_mirror_direction_set(rng: &mut impl Rng, value: &Value) -> Vec<Vec<MirrorLine>> {
    let directions = value
        .get("directions").expect("[mirror.directions] must specify at least one direction.")
        .as_sequence().expect("[mirror.directions] must be a list.");

    let directions = directions.iter().map(|direction_set| {
        let direction_set = direction_set.as_sequence().expect("[mirror.directions[$]] should be a sequence of strings.");
        direction_set.iter().map(|entry| {
            entry.as_str().expect("[mirror.directions[$][$]] must be a string.")
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut mirror_lines = vec![];

    for direction_set in directions {
        let mut parsed_set = vec![];

        for direction in direction_set {
            parsed_set.push(match direction {
                "downright" => MirrorLine::Downright,
                "upright" => MirrorLine::Downright,
                "horizontal" => MirrorLine::Downright,
                "vertical" => MirrorLine::Downright,
                _ => panic!("[mirror.directions[$].direction] found invalid direction [{direction}]"),
            });
        }

        mirror_lines.push(parsed_set);
    }

    mirror_lines
}

pub fn parse_increase_strategy(rng: &mut impl Rng, value: &Value, strategy: &str) -> Increase {
    let increase_strategy = value.get("increase-strategy").expect(format!("[ordered.strategy ({strategy})] requires [ordered.increase-strategy]").as_str())
        .as_mapping().expect("[ordered.increase-strategy] must be a mapping.");

    let strategy_type = increase_strategy.get("type").expect("[ordered.strategy.increase-strategy] must have a [type]");

    let factor = parse_u64_param(rng, increase_strategy.get("factor").expect("[ordered.strategy.increase-strategy] must have a [factor]"));

    match strategy_type {
        Value::Mapping(mapping) => {
            let linear = mapping.get("linear").map(|ratio| ratio.as_f64().expect(format!("[ordered.orientation.horizontal] must be a float.").as_str()));
            let exponential = mapping.get("exponential").map(|ratio| ratio.as_f64().expect(format!("[ordered.orientation.vertical] must be a float.").as_str()));

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
    }
}

pub fn parse_matrix_size(rng: &mut impl Rng, value: &Value, strategy: &str) -> u64 {
    let matrix_size = value.get("matrix-size").expect(format!("[ordered.strategy ({strategy})] requires [ordered.matrix-size]").as_str());
    parse_u64_param(rng, matrix_size)
}

pub fn parse_orientation(rng: &mut impl Rng, value: &Value, strategy: &str) -> Orientation {
    let orientation = value.get("orientation").expect(format!("[ordered.strategy ({strategy})] requires [ordered.orientation]").as_str());

    match orientation {
        Value::Mapping(mapping) => {
            let horizontal_ratio = mapping.get("horizontal").map(|ratio| ratio.as_f64().expect(format!("[ordered.orientation.horizontal] must be a float.").as_str()));
            let vertical_ratio = mapping.get("vertical").map(|ratio| ratio.as_f64().expect(format!("[ordered.orientation.vertical] must be a float.").as_str()));

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
    }
}

pub fn parse_diagonaldirection(rng: &mut impl Rng, value: &Value, strategy: &str) -> DiagonalDirection {
    let diagonaldirection = value.get("diagonal-direction").expect(format!("[ordered.strategy ({strategy})] requires [ordered.diagonal-direction]").as_str());

    match diagonaldirection {
        Value::Mapping(mapping) => {
            let dr_ratio = mapping.get("down-right").map(|ratio| ratio.as_f64().expect(format!("[ordered.orientation.horizontal] must be a float.").as_str()));
            let ur_ratio = mapping.get("up-right").map(|ratio| ratio.as_f64().expect(format!("[ordered.orientation.vertical] must be a float.").as_str()));

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
    }
}

// end