use rand::Rng;
use serde_yaml::Value;

use crate::{
    effects::{BaseResult, Log},
    parsers::util::{logless::parse_property_as_f64, parse_property_as_u64_complex},
};

pub fn parse_chance(log: Log, value: &Value) -> BaseResult<f64> {
    Ok(
        parse_property_as_f64(value, "chance", Some(0.0))
            .expect("[mirror.chance] must be a float."),
    )
}

pub fn process_chance(log: Log, rng: &mut impl Rng, value: &Value) -> BaseResult<bool> {
    let chance = parse_chance(log, value)?;
    Ok(rng.gen_range(0.0..1.0) < chance)
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
