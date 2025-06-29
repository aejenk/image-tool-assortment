use image_effects::dither::ordered::algorithms::properties::Increase;
use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::util::parse_property_as_u64_complex,
};

pub fn parse_increase_strategy(
    log: Log,
    rng: &mut impl Rng,
    value: &Value,
) -> BaseResult<Increase> {
    let increase_strategy = value
        .get("increase-strategy")
        .unwrap_or_else(|| panic!("[ordered.strategy] requires [ordered.increase-strategy]"));

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
