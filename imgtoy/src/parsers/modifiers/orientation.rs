use image_effects::dither::ordered::algorithms::properties::Orientation;
use rand::Rng;
use serde_yaml::Value;

use crate::effects::{BaseResult, Log};

pub fn parse_orientation(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<Orientation> {
    let orientation = value
        .get("orientation")
        .unwrap_or_else(|| panic!("[ordered.strategy] requires [ordered.orientation]"));

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
