use image_effects::dither::ordered::algorithms::properties::DiagonalDirection;
use rand::Rng;
use serde_yaml::Value;

use crate::effects::{BaseResult, Log};

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
